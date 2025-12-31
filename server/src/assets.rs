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

        let asset_path = ctx.db.assets()
            .insert(AssetFile { path, value }).path;

        // Is scripts?
        if asset_path.ends_with(".rn") {
            ctx.db.scripts().insert(ScriptAsset { asset_path });
        }
    }
}

pub fn load_scripts(ctx: &ReducerContext) {
    shared::clear_scripts();
    let scripts: Vec<ScriptAsset> = ctx.db.scripts().iter().collect();

    for i in 0..scripts.len() {
        let assets = ctx.db.assets().path().find(&scripts[i].asset_path).unwrap();
        let data = String::from_utf8(assets.value).unwrap();

        if let Err(e) = shared::insert_script(i as u32, data) {
            log::error!("Script insertion error: {}", e);
        }
    }
}