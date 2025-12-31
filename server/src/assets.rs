use std::{collections::*, path::Path};
use spacetimedb::{ReducerContext, Table};

#[spacetimedb::table(name=assets)]
pub struct AssetFile {
    #[primary_key]
    path: String,
    value: Vec<u8>
}

#[spacetimedb::table(name=scripts)]
pub struct ScriptAsset {
    #[primary_key]
    asset_path: String
}

pub fn load_assets(ctx: &ReducerContext) {
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

        let asset_path = ctx.db.assets()
            .insert(AssetFile { path, value }).path;

        // Is script?
        if asset_path.ends_with(".rn") {
            ctx.db.scripts().insert(ScriptAsset { asset_path });
        }
    }
}