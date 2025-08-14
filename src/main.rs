use std::path::PathBuf;
use bevy::{
    prelude::*,
    platform::collections::*,
    asset::io::embedded::EmbeddedAssetRegistry
};
use bevy_spacetimedb::*;

mod stdb;
use stdb::*;
use spacetimedb_sdk::*;

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<stdb::DbConnection>>;

// utils

impl stdb::StIVec3 {
    fn _into(self) -> IVec3 { 
        IVec3::new(self.x, self.y, self.z)
    }
}

impl stdb::StVec3 {
    fn into(self) -> Vec3 { 
        Vec3::new(self.x, self.y, self.z)
    }
}

// Components

#[derive(Component)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub identity: Identity,
    pub position: Vec3,
}

// Resources

#[derive(Resource, Default)]
/// Server stats
pub struct TicksInfo {
    // Current tick
    pub tick: u128,
    // Difference between previous tick
    pub tickrate: f64,
}

#[derive(Resource, Default)]
pub struct TexturesHandler(HashMap<u16, Option<Handle<Image>>>);

// Systems

fn subscribe_to_main(ctx: &SubscriptionEventContext) {
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
            "SELECT * FROM chunk",
            "SELECT * FROM ticks",
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
    mut events: ReadInsertEvent<stdb::StAsset>,
    assets: Res<EmbeddedAssetRegistry>,
) {
    for event in events.read() {
        info!("Inserted asset: {}", event.row.name);

        let full = PathBuf::from("");
        let relative = PathBuf::from(event.row.name.clone());

        assets.insert_asset(full, &relative, event.row.value.clone());
    }
}

fn on_asset_updated(
    mut events: ReadUpdateEvent<stdb::StAsset>,
    assets: Res<EmbeddedAssetRegistry>,
) {
    for event in events.read() {
        info!("Updated asset: {}", event.new.name);

        let full = PathBuf::from("");
        let relative = PathBuf::from(event.new.name.clone());

        assets.insert_asset(full, &relative, event.new.value.clone());
    }
}

pub struct Model;
impl Model {
    pub fn load(block: &Block, assets: &AssetServer) -> Option<Handle<Image>>{
        match &block.model {
            ModelType::Cube(path) => {
                Some(assets.load(format!("embedded://{}", path)))
            },
            _ => None
        }
    }
}

fn on_block_inserted(
    mut events: ReadInsertEvent<stdb::Block>,
    mut textures: ResMut<TexturesHandler>,
    assets: Res<AssetServer>,
) {
    for event in events.read() {
        info!("Inserted Block({}): {:?}", event.row.id, event.row.model);

        let texture = Model::load(&event.row, &assets);

        textures.0.insert(event.row.id, texture);
    }
}

fn on_player_inserted(
    mut events: ReadInsertEvent<stdb::Player>,
    mut commands: Commands
) {
    for event in events.read() {
        commands.spawn(Player {
            id: event.row.id,
            name: event.row.name.clone(),
            identity: event.row.identity,
            position: event.row.position.clone().into(),
        });
    }
}

fn on_chunk_inserted(
    mut events: ReadInsertEvent<stdb::Chunk>,
) {
    for _event in events.read() {

    }
}

fn on_mesh_inserted(
    mut events: ReadInsertEvent<stdb::Mesh>,
) {
    for _event in events.read() {
        
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

fn main() {
    App::new()
        .add_plugins(
            StdbPlugin::default()
                .with_uri("http://localhost:3000")
                .with_module_name("morph")
                .with_run_fn(stdb::DbConnection::run_threaded)
                .add_table(stdb::RemoteTables::player)
                .add_table(stdb::RemoteTables::asset)
                .add_table(stdb::RemoteTables::block)
                .add_table(stdb::RemoteTables::chunk)
                .add_table(stdb::RemoteTables::ticks)
                .add_table(stdb::RemoteTables::mesh)
            )
        .add_plugins(DefaultPlugins)
        .init_resource::<TicksInfo>()
        .init_resource::<TexturesHandler>()
        .add_systems(
            FixedPostUpdate, (
            on_connected,
            on_player_inserted,
            on_asset_inserted,
            on_asset_updated,
            on_block_inserted,
            on_chunk_inserted,
            on_mesh_inserted,
            on_ticks_updated
        ))
        .run();
}
