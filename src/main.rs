use std::path::PathBuf;
use bevy::{
    prelude::*,
    platform::{
        collections::*,
        sync::atomic::*,
        sync::*,
    },
    asset::io::embedded::EmbeddedAssetRegistry
};
use bevy_spacetimedb::*;

mod stdb;
use spacetimedb_sdk::{Table, TableWithPrimaryKey};
use stdb::*;

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

#[derive(Resource, Clone, Default)]
pub struct AssetsQueue(Arc<RwLock<HashSet<u64>>>);

#[derive(Resource, Clone, Default)]
pub struct NeedReload(Arc<AtomicBool>);

impl NeedReload {
    pub fn update(&self, value: bool) {
        self.0.store(value, Ordering::Relaxed);
    }

    pub fn get(&self) -> bool {
        self.0.swap(false, Ordering::SeqCst)
    }
}

fn main() {
    App::new()
        .init_resource::<AssetsQueue>()
        .init_resource::<NeedReload>()
        .add_plugins(
            StdbPlugin::default()
                .with_uri("http://localhost:3000")
                .with_module_name("morph")
                .with_run_fn(DbConnection::run_threaded)
            )
        .add_plugins(DefaultPlugins)
        .add_systems(PreStartup, setup)
        .add_systems(FixedPostUpdate, (queue_assets, hot_reload).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    need_reload: Res<NeedReload>,
    assets_queue: Res<AssetsQueue>,
    handler: SpacetimeDB,
) {
    let need_reload = need_reload.into_inner();
    let assets = assets_queue.into_inner();
    commands.spawn(Camera3d::default());


    let reloader = need_reload.clone();
    handler.db().block().on_insert(move |_, _| {
        reloader.update(true);
    });

    let reloader = need_reload.clone();
    handler.db().block().on_update(move |_, _, _| {
        reloader.update(true);
    });

    handler.db().mesh().on_insert(move |_, mesh| {
        println!("Inserted Mesh({:?}) = Vertices({})", mesh.position, mesh.vertices.len());
    });

    handler.db().chunk().on_insert(move |_, chunk| {
        println!("Inserted Chunk({:?})", chunk.position);
    });

    let files = assets.clone();
    handler.db().asset().on_insert(move |_, asset| {
        let mut access = files.0.write().unwrap();
        access.insert(asset.id);
    });

    let files = assets.clone();
    handler.db().asset().on_update(move |_, _, asset| {
        let mut access = files.0.write().unwrap();
        access.insert(asset.id);
    });

    handler.subscription_builder()
        .on_applied(|_ctx| {
            println!("Successful subscription!");
        })
        .on_error(|_ctx, err| {
            eprintln!("Subcribe error: {}", err);
        })
        .subscribe([
            "SELECT * FROM player",
            "SELECT * FROM block",
            "SELECT * FROM asset",
            "SELECT * FROM chunk",
            "SELECT * FROM mesh",
        ]);
}

fn queue_assets(
    assets_queue: Res<AssetsQueue>,
    handler: SpacetimeDB,
    assets: Res<EmbeddedAssetRegistry>,
) {
    let mut access = assets_queue.0.write().unwrap();
    for id in access.drain() {
        let asset = handler.db().asset().id().find(&id).unwrap();
        println!("Queued asset: {}", asset.name);

        let full = PathBuf::from("");
        let relative = PathBuf::from(asset.name);

        assets.insert_asset(full, &relative, asset.value);
    }
}

// todo: load blocks and textures from server
// update all blocks / meshes / assets data 
fn hot_reload(
    need_reload: Res<NeedReload>,
    handler: SpacetimeDB,
) {
    if need_reload.get() {
        // todo: update textures data
        // todo: update meshes data
        for block in handler.db().block().iter() {
            println!("id: {}", block.id);
        }

    }
}