use std::path::PathBuf;
use bevy::{
    prelude::*,
    platform::{
        collections::*,
        sync::*
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

fn main() {
    App::new()
        .init_resource::<AssetsQueue>()
        .add_plugins(
            StdbPlugin::default()
                .with_uri("http://localhost:3000")
                .with_module_name("morph")
                .with_run_fn(DbConnection::run_threaded)
            )
        .add_plugins(DefaultPlugins)
        .add_systems(PreStartup, setup)
        .add_systems(FixedPostUpdate, queue_assets)
        .run();
}

fn setup(
    mut commands: Commands,
    assets_queue: Res<AssetsQueue>,
    handler: SpacetimeDB,
) {
    let assets = assets_queue.into_inner();
    commands.spawn(Camera3d::default());

    handler.db().chunk().on_insert(move |_, chunk| {
        println!("Inserted Chunk({:?}) = {} bytes", chunk.position, chunk.data.len());
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