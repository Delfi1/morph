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
use spacetimedb_sdk::Table;
use stdb::*;

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

#[derive(Resource, Clone, Default)]
pub struct FilesQueue(Arc<RwLock<HashSet<u64>>>);

fn main() {
    App::new()
        .init_resource::<FilesQueue>()
        .add_plugins(
            StdbPlugin::default()
                .with_uri("http://localhost:3000")
                .with_module_name("morph")
                .with_run_fn(DbConnection::run_threaded)
            )
        .add_plugins(DefaultPlugins)
        .add_systems(PreStartup, setup)
        .add_systems(FixedPostUpdate, queue_files)
        .run();
}

fn setup(
    mut commands: Commands,
    files_queue: Res<FilesQueue>,
    handler: SpacetimeDB,
) {
    commands.spawn(Camera3d::default());

    let file_queue = files_queue.into_inner().clone();
    handler.db().file().on_insert(move |_, file| {
        let mut access = file_queue.0.write().unwrap();
        access.insert(file.id);
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
            "SELECT * FROM chunk",
            "SELECT * FROM mesh",
            "SELECT * FROM file",
        ]);
}

fn queue_files(
    files_queue: Res<FilesQueue>,
    handler: SpacetimeDB,
    assets: Res<EmbeddedAssetRegistry>,
) {
    let mut access = files_queue.0.write().unwrap();
    for id in access.drain() {
        let file = handler.db().file().id().find(&id).unwrap();
        println!("Queue file: {}", file.name);

        let full = PathBuf::from("");
        let relative = PathBuf::from(file.name);

        assets.insert_asset(full, &relative, file.value);
    }
}