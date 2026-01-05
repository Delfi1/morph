use std::collections::*;
use spacetimedb::{ReducerContext, Table};

#[spacetimedb::table(name=assets, public)]
pub struct AssetFile {
    #[primary_key]
    pub path: String,
    pub value: Vec<u8>,
    
    /// File's hash key
    pub digest: Vec<u8>
}

#[spacetimedb::table(name=scripts, public)]
pub struct ScriptAsset {
    #[primary_key]
    pub asset_path: String
}

/// On asset changer
fn update_asset(ctx: &ReducerContext, asset: AssetFile) {
    let Some(format) = asset.path.split('.').last() else { return };

    match format {
        "rn" => {
            let data = String::from_utf8(asset.value).unwrap();
            ctx.db.scripts().insert(ScriptAsset { asset_path: asset.path.clone() });

            if let Err(e) = shared::insert_script(asset.path, data) {
                log::error!("Script insertion error: {}", e);
            }
        },

        // TODO: other formats
        _ => ()
    }
}

/// On asset changer
fn remove_asset(ctx: &ReducerContext, path: String) {
    let Some(format) = path.split('.').last() else { return };

    match format {
        "rn" => {
            ctx.db.scripts().asset_path().delete(&path);

            shared::remove_script(&path);
        },

        // TODO: other formats
        _ => ()
    }
}

/// Insert new asset to DB or update it
pub fn add_raw_asset(ctx: &ReducerContext, path: String, value: Vec<u8>) {
    let digest = shared::assets::digest(&value);
    log::info!("Updated asset: {}", path);

    let asset = AssetFile { path, value, digest };

    // Insert or update asset data 
    let asset = match ctx.db.assets().path().find(&asset.path).is_none() { 
        true => ctx.db.assets().insert(asset),
        false => ctx.db.assets().path().update(asset)
    };

    update_asset(ctx, asset);
}

/// Remove asset from DB
pub fn remove_raw_asset(ctx: &ReducerContext, path: String) {
    ctx.db.assets().path().delete(&path);

    remove_asset(ctx, path);
}

pub fn init(ctx: &ReducerContext) {
    let old_keys = ctx.db.assets().iter()
        .map(|s| s.path).collect::<HashSet<String>>();

    let values = shared::assets::load_assets();
    let keys = values.keys().cloned().collect::<HashSet<String>>();

    // Removed assets
    for key in old_keys.difference(&keys) {
        ctx.db.assets().path().delete(key);
        ctx.db.scripts().asset_path().delete(key);
    }

    // Other rows
    for (path, value) in values {
        ctx.db.assets().path().delete(&path);
        ctx.db.scripts().asset_path().delete(&path);
        let digest = shared::assets::digest(&value);

        let file = ctx.db.assets().insert(AssetFile { path, value, digest });
        update_asset(ctx, file);
    }
}
