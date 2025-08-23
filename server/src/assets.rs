use include_directory::{Dir, include_directory};
use log::debug;
use serde::de::DeserializeOwned;
use spacetimedb::{ReducerContext, Table, table};

static ASSETS_DIR: Dir<'_> = include_directory!("./assets");

#[table(name = asset, public)]
pub struct StAsset {
    #[auto_inc]
    #[primary_key]
    pub id: u64,
    #[unique]
    pub name: String,
    pub value: Vec<u8>,
}

pub fn load_assets(ctx: &ReducerContext) {
    debug!("Loading assets from ./assets/ ...");

    for file in ASSETS_DIR.files() {
        let name = file
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let value = file.contents().to_vec();

        if let Some(mut dbfile) = ctx.db.asset().name().find(name.clone()) {
            if dbfile.value != value {
                dbfile.value = value;
                ctx.db.asset().id().update(dbfile);
            }
        } else {
            ctx.db.asset().insert(StAsset { id: 0, name, value });
        }
    }
}

pub fn insert_or_update_asset_bytes(ctx: &ReducerContext, name: &str, bytes: Vec<u8>) {
    if let Some(mut dbfile) = ctx.db.asset().name().find(name.to_string()) {
        if dbfile.value != bytes {
            dbfile.value = bytes;
            ctx.db.asset().id().update(dbfile);
        }
    } else {
        ctx.db.asset().insert(StAsset {
            id: 0,
            name: name.to_string(),
            value: bytes,
        });
    }
}

pub fn ensure_schema_assets(ctx: &ReducerContext) {
    for file in crate::SCHEME_DIR.files() {
        let name = file
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let value = file.contents().to_vec();
        insert_or_update_asset_bytes(ctx, &name, value);
    }
}

pub fn get_json_asset<T: DeserializeOwned>(
    ctx: &ReducerContext,
    name: &str,
    default_json: &str,
) -> T {
    if let Some(dbfile) = ctx.db.asset().name().find(name.to_string()) {
        return serde_json::from_slice(&dbfile.value)
            .expect(&format!("Invalid JSON in asset (DB): {}", name));
    }

    if let Some(file) = crate::SCHEME_DIR.get_file(name) {
        let bytes = file.contents().to_vec();
        insert_or_update_asset_bytes(ctx, name, bytes.clone());
        return serde_json::from_slice(&bytes)
            .expect(&format!("Invalid JSON in schema file: {}", name));
    }

    serde_json::from_str(default_json).expect(&format!("Invalid default JSON for asset {}", name))
}
