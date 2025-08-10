use bevy::prelude::*;
use bevy_spacetimedb::*;

mod stdb;
use stdb::*;

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

fn main() {
    App::new()
        .add_plugins(
            StdbPlugin::default()
                .with_uri("http://localhost:3000")
                .with_module_name("morph")
                .with_run_fn(DbConnection::run_threaded)
            )
        .add_plugins(DefaultPlugins)
        .add_systems(PostStartup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    handler: SpacetimeDB,
) {
    commands.spawn(Camera3d::default());
    
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