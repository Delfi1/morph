// todo: chunks, blocks metadata, client-side rendering,
// server-side mesher, and meshes cache
use log::debug;
use spacetimedb::{
    reducer, ReducerContext, Table
};

pub mod math;
mod player;
mod assets;
mod chunks;
mod mesher;

use assets::file;

#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    // generate server assets
    assets::load_assets(&ctx);

    debug!("Total assets: {}", ctx.db.file().count());

    // create blocks data
    chunks::init_blocks(&ctx);

    Ok(())
}

#[reducer(client_connected)]
pub fn identity_connected(_ctx: &ReducerContext) {

}

#[reducer(client_disconnected)]
pub fn identity_disconnected(_ctx: &ReducerContext) {

}
