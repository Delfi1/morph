use std::path::*;
use bevy::{
    app::*,
    prelude::*,
    render::primitives::*,
    platform::collections::*, 
    asset::io::embedded::EmbeddedAssetRegistry,
};
use bevy_spacetimedb::*;
use spacetimedb_sdk::*;

use super::stdb::{
    self,
    PlayerTableAccess,
    MeshTableAccess,
    ChunkTableAccess,
    BlockTableAccess,
    AssetTableAccess,
    TicksTableAccess
};

use super::camera;
use super::renderer;
use super::utils::*;

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<stdb::DbConnection>>;

#[derive(Component)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub identity: Identity,
}

#[derive(Component)]
// Current player marker
pub struct CurrentPlayer;

#[derive(Resource, Default)]
pub struct PlayersHandler(HashMap<u64, Entity>);

#[derive(Resource, Default)]
pub struct MeshesHandler(HashMap<u64, Entity>);

#[derive(Resource, Default)]
/// Server stats
pub struct TicksInfo {
    // Current tick
    pub tick: u128,
    // Difference between previous tick
    pub tickrate: f64,
}

fn subscribe_to_main(ctx: &stdb::SubscriptionEventContext) {
    info!("Subscibed to assets...");

    // Subscribe to other tables
    ctx.subscription_builder()
        .on_applied(|_| {
            info!("Succesful subscription!");
        })
        .on_error(|_, err| {
            error!("Subcribe error: {}", err);
        })
        .subscribe([
            "SELECT * FROM player",
            "SELECT * FROM block",
            "SELECT * FROM ticks",
            // todo: subsribe when joined
            "SELECT * FROM mesh",  
        ]);
}

fn on_connected(
    mut events: ReadStdbConnectedEvent,
    handler: SpacetimeDB,
) {
    for _ in events.read() {
        // Subscribe to assets
        handler.subscription_builder()
            .on_applied(subscribe_to_main)
            .on_error(|_, err| {
                error!("Subcribe error: {}", err);
            })
            .subscribe(["SELECT * FROM asset"]);

        info!("Players count: {}", handler.db().player().count());
    }
}

fn on_asset_inserted(
    mut commands: Commands,
    mut events: ReadInsertEvent<stdb::StAsset>,
    embedded_assets: Res<EmbeddedAssetRegistry>,
) {
    for event in events.read() {
        info!("Inserted asset: {}", event.row.name);

        let full = PathBuf::from("");
        let relative = Path::new(&event.row.name);

        embedded_assets.insert_asset(full, &relative, event.row.value.clone());

        // Insert required assets:
        if event.row.name.eq("chunk.wgsl") {
            commands.insert_resource(renderer::ChunkShaderLoader("embedded://chunk.wgsl"));
        }
    }
}

fn on_asset_updated(
    mut commands: Commands,
    mut events: ReadUpdateEvent<stdb::StAsset>,
    embedded_assets: Res<EmbeddedAssetRegistry>,
) {
    if events.is_empty() { return }

    for event in events.read() {
        info!("Updated asset: {}", event.new.name);

        let full = PathBuf::from("");
        let relative = Path::new(&event.new.name);

        embedded_assets.insert_asset(full, &relative, event.new.value.clone());

        // Update required assets:
        if event.new.name.eq("chunk.wgsl") {
            commands.insert_resource(renderer::ChunkShaderLoader("embedded://chunk.wgsl"));
        }
    }

    // todo: hot-reload morph world
}

fn on_block_inserted(
    mut commands: Commands,
    handler: SpacetimeDB,
    mut events: ReadInsertEvent<stdb::Block>,
) {
    if events.is_empty() { return }
    events.clear();

    let l = handler.db().block().count() as usize;
    let mut blocks = HashMap::with_capacity(l);

    for block in handler.db().block().iter() {
        blocks.insert(block.id, block);
    }

    commands.insert_resource(renderer::LoadHandler(blocks));
}

fn on_player_inserted(
    mut events: ReadInsertEvent<stdb::Player>,
    mut players: ResMut<PlayersHandler>,
    handler: SpacetimeDB,
    mut commands: Commands
) {
    for event in events.read() {
        let position = event.row.position.clone().into();
        let mut player = commands.spawn(Player {
            id: event.row.id,
            name: event.row.name.clone(),
            identity: event.row.identity,
        });
        player.insert(Transform::from_translation(position));

        if handler.identity() == event.row.identity {
            player.insert((
                CurrentPlayer,
                Camera3d::default(),
                Transform::from_xyz(0.0, 4.0, 0.0),
                camera::MainCamera::new()
            ));
        } else {
            // todo: test player model (sphere) and nickname text
            //player.insert()
        }

        players.0.insert(event.row.id, player.id());
    }
}

fn on_player_updated(
    mut events: ReadUpdateEvent<stdb::Player>,
    mut commands: Commands,
    players: Res<PlayersHandler>
) {
    for event in events.read() {
        let position = event.new.position.clone().into();
        let entity = players.0.get(&event.new.id).unwrap();
        commands.entity(*entity).insert(Transform::from_translation(position));
    }
}

fn on_player_deleted(
    mut events: ReadDeleteEvent<stdb::Player>,
    mut commands: Commands,
    mut players: ResMut<PlayersHandler>
) {
    for event in events.read() {
        let entity = players.0.remove(&event.row.id).unwrap();
        commands.entity(entity).despawn();
    }
}

fn on_chunk_inserted(
    mut events: ReadInsertEvent<stdb::Chunk>,
) {
    for _event in events.read() {

    }
}

fn on_mesh_inserted(
    mut commands: Commands,
    mut handler: ResMut<MeshesHandler>,
    mut events: ReadInsertEvent<stdb::Mesh>,
) {
    for event in events.read() {
        if event.row.vertices.is_empty() { continue; }
        let position: IVec3 = event.row.position.clone().into();
        let global = position.as_vec3() * Vec3::splat(SIZE_F32);

        info!("Inserted mesh: {}", position);

        let vertices = event.row.vertices.clone();
        let indices = event.row.indices.clone();

        let id = commands.spawn((
            Visibility::default(),
            Aabb::from_min_max(
                Vec3::splat(-SIZE_F32),
                Vec3::splat(SIZE_F32 * 2.0),
            ),
            renderer::ChunkMesh::new(global, vertices, indices),
            Transform::from_translation(global)
        )).id();

        handler.0.insert(event.row.id, id);
    }
}

fn on_mesh_updated(
    mut events: ReadUpdateEvent<stdb::Mesh>,
) {
    for _event in events.read() {
        //todo: update mesh entity 
    }
}

fn on_ticks_updated(
    mut events: ReadUpdateEvent<stdb::Ticks>,
    mut ticks_info: ResMut<TicksInfo>
) {
    for event in events.read() {
        ticks_info.tick = event.new.tick;
        ticks_info.tickrate = event.new.tickrate;
    }
}

#[derive(Default)]
pub struct NetworkingPlugin;
impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StdbPlugin::default()
            .with_uri("http://localhost:3000")
            .with_module_name("morph")
            .with_run_fn(stdb::DbConnection::run_threaded)
            .add_table(stdb::RemoteTables::player)
            .add_table(stdb::RemoteTables::asset)
            .add_table(stdb::RemoteTables::block)
            .add_table(stdb::RemoteTables::chunk)
            .add_table(stdb::RemoteTables::ticks)
            .add_table(stdb::RemoteTables::mesh)
        );

        app.init_resource::<TicksInfo>()
            .init_resource::<PlayersHandler>()
            .init_resource::<MeshesHandler>();

        app.add_systems(FixedPostUpdate, (
            on_connected,
            on_player_inserted,
            on_player_updated,
            on_player_deleted,
            on_asset_inserted,
            on_asset_updated,
            on_block_inserted,
            on_chunk_inserted,
            on_mesh_inserted,
            on_mesh_updated,
            on_ticks_updated,
        ));
    }
}