use log::info;
use spacetimedb::{table, ReducerContext, Table};
use include_directory::{include_directory, Dir};

static ASSETS_DIR: Dir<'_> = include_directory!("./assets");

// Assets files data
#[table(name = file, public)]
pub struct File {
    #[auto_inc]
    #[primary_key]
    id: u64,
    #[unique]
    name: String,
    value: Vec<u8>
}

// todo: create assets trees
// todo: create recursive read (load) assets function

pub fn load_assets(ctx: &ReducerContext) {
    info!("Loading assets...");

    for file in ASSETS_DIR.files() {
        let path = file.path();
        let name = String::from(
            path.file_name().unwrap().to_str().unwrap()
        );
        let value = file.contents().to_vec();

        if let Some(mut dbfile) = ctx.db.file().name().find(&name) {
            dbfile.value = value;
            
            ctx.db.file().id().update(dbfile);
            continue;
        }

        ctx.db.file().insert(File {
            id: 0,
            name,
            value
        });
    }

}