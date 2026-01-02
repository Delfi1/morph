// Todo: syncer can load files from server and disk to sync data

use std::collections::*;
use syncer::networking::*;
use bevy_spacetimedb::*;
use std::path::PathBuf;
use bevy::{
    asset::*, 
    prelude::*
};

pub type SpacetimeDB<'a> = Res<'a, StdbConnection<DbConnection>>;

#[derive(Asset, TypePath, Debug)]
// Asset container
struct Blob {
    path: PathBuf,
    value: Vec<u8>,
}

#[derive(Default, TypePath)]
struct BlobAssetLoader;

impl AssetLoader for BlobAssetLoader {
    type Asset = Blob;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn io::Reader,
        _: &(),
        ctx: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut value = Vec::new();

        let path = PathBuf::from(ctx.path());
        reader.read_to_end(&mut value).await?;

        Ok(Blob { path, value })
    }
}

fn on_connected(
    mut messages: ReadStdbConnectedMessage,
    stdb: SpacetimeDB,
) {
    for _ in messages.read() {
        stdb.subscription_builder()
            .on_applied(|_| info!("Subscription applied"))
            .on_error(|_, err| error!("Subscription failed for: {}", err))
            .subscribe(["SELECT * FROM assets", "SELECT * FROM scripts"]);
    }
}

fn on_assets(
    mut inserted: ReadInsertMessage<AssetFile>,
    mut updated: ReadUpdateMessage<AssetFile>,
    mut deleted: ReadDeleteMessage<AssetFile>, 
) {
    for message in inserted.read() {
        std::fs::write(
            format!("assets/{}", message.row.path), 
            &message.row.value
        ).expect("File write error");
    }

    for message in updated.read() {
        std::fs::write(
            format!("assets/{}", message.new.path), 
            &message.new.value
        ).expect("File write error");
    }

    for message in deleted.read() {
        if !std::fs::exists(&message.row.path).unwrap() { continue; }

        std::fs::remove_file(
            format!("assets/{}", message.row.path), 
        ).expect("File remove error")
    }
}

#[derive(Resource, Default)]
pub struct AssetsHandler(HashMap<PathBuf, Handle<Blob>>);

fn reload_assets(
    stdb: SpacetimeDB,
    mut events: MessageReader<AssetEvent<Blob>>,
    assets: Res<Assets<Blob>>,
    asset_server: Res<AssetServer>,
    mut handler: ResMut<AssetsHandler>
) {
    let new: HashSet<PathBuf> = shared::assets::assets_paths("./assets");
    let old: HashSet<PathBuf> = handler.0.keys().cloned().collect();

    // Removed files
    for path in old.difference(&new) {
        let relative = PathBuf::from_iter(path.components().skip(2));
        let path = relative.to_str().unwrap().to_string();

        stdb.reducers().remove_asset(path).unwrap();
    }

    for path in new.difference(&old) {
        let relative = PathBuf::from_iter(path.components().skip(2));

        handler.0.insert(path.clone(), asset_server.load(relative));
    }

    for message in events.read() {
        match message {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id } => {
                let handler = assets.get(id.clone()).unwrap();
                let path = handler.path.to_str().unwrap().to_string();
                let digest = shared::assets::digest(&handler.value);

                // Skip if not overrided
                if let Some(asset) = stdb.db().assets().path().find(&path) {
                    if asset.digest == digest { continue; }
                }

                info!("Reload: {}", path);
                stdb.reducers().edit_asset(path, handler.value.clone()).unwrap();
            },
            _ => ()
        }
    }
}

/// Conntect to server, watch files, etc
fn main() {
    App::new()
        .add_plugins((MinimalPlugins, bevy::log::LogPlugin::default()))
        .add_systems(FixedPostUpdate, (on_connected, on_assets, reload_assets).chain())
        .add_plugins(AssetPlugin {
            file_path: "../assets".to_string(),
            ..default()
        })
        .init_asset_loader::<BlobAssetLoader>()
        .init_asset::<Blob>()
        .init_resource::<AssetsHandler>()
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

