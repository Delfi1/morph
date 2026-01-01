use std::collections::*;
use spacetimedb::{ReducerContext, Table};

#[spacetimedb::table(name=assets)]
pub struct AssetFile {
    #[primary_key]
    pub path: String,
    pub value: Vec<u8>
}

#[spacetimedb::table(name=scripts)]
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

/// Insert new asset to DB or update it
pub fn add_raw_asset(ctx: &ReducerContext, path: String, value: Vec<u8>) {
    let asset = match ctx.db.assets().path().find(&path).is_none() { 
        true => ctx.db.assets().insert(AssetFile { path, value }),
        false => ctx.db.assets().path().update(AssetFile {path, value } )
    };

    update_asset(ctx, asset);
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

        let file = ctx.db.assets().insert(AssetFile { path, value });
        update_asset(ctx, file);
    }
}
