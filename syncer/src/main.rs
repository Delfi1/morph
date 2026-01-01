// Todo: syncer can load files from server and disk to sync data

use std::{fmt::format, io::Write};

use syncer::networking::*;
use bevy_spacetimedb::*;
use spacetimedb_sdk::*;
use bevy::prelude::*;

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

fn on_connected(
    mut messages: ReadStdbConnectedMessage,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for _ in messages.read() {
        info!("Connected to SpacetimeDB");

        stdb.subscription_builder()
            .on_applied(|_| info!("Subscription applied"))
            .on_error(|_, err| error!("Subscription failed for: {}", err))
            .subscribe(["SELECT * FROM assets", "SELECT * FROM scripts"]);

        info!("Assets count: {}", stdb.db().assets().count());
    }
}

fn on_assets(
    mut inserted: ReadInsertMessage<AssetFile>,
    mut updated: ReadUpdateMessage<AssetFile>,
    mut deleted: ReadDeleteMessage<AssetFile>, 
) {

}

/// Conntect to server, watch files, etc
fn main() {
    App::new()
        .add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()))
        .add_systems(PostUpdate, (on_connected, on_assets))
        .add_plugins(AssetPlugin::default())
        .add_plugins(
            StdbPlugin::default()
                .with_uri(syncer::URI)
                .with_module_name(syncer::MODULE)
                .with_run_fn(DbConnection::run_threaded)
                .add_table(RemoteTables::assets)
                .add_table(RemoteTables::scripts)
        )
        .run();
}

