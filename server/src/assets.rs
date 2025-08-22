use image::{DynamicImage, GenericImageView, load_from_memory};
use include_directory::{Dir, include_directory};
use log::debug;
use spacetimedb::{ReducerContext, Table, table};

static ASSETS_DIR: Dir<'_> = include_directory!("./assets");

// Assets data
#[table(name = asset, public)]
pub struct StAsset {
    #[auto_inc]
    #[primary_key]
    id: u64,
    #[unique]
    name: String,
    value: Vec<u8>,
}

// todo: create assets trees
// todo: create recursive read (load) assets function

pub fn load_assets(ctx: &ReducerContext) {
    debug!("Loading assets...");

    for file in ASSETS_DIR.files() {
        let path = file.path();
        let name = String::from(path.file_name().unwrap().to_str().unwrap());
        let value = file.contents().to_vec();

        if let Some(mut dbfile) = ctx.db.asset().name().find(&name) {
            dbfile.value = value;

            ctx.db.asset().id().update(dbfile);
            continue;
        }

        ctx.db.asset().insert(StAsset { id: 0, name, value });
    }
}

/// Загружает картинку из ассетов по имени
pub fn load_asset_image(ctx: &ReducerContext, asset_name: &str) -> Option<DynamicImage> {
    ctx.db
        .asset() // метод asset() существует благодаря #[table(name = asset)]
        .iter()
        .find(|a| a.name == asset_name)
        .map(|asset| {
            load_from_memory(&asset.value)
                .expect(&format!("Не удалось декодировать PNG {}", asset_name))
        })
}
