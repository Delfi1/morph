// todo: chunks, blocks metadata, client-side rendering,
// server-side mesher, and meshes cache
use log::info;
use spacetimedb::{
    reducer, ReducerContext, Table
};

pub mod math;
mod player;
mod assets;
mod chunks;
mod mesher;

use assets::*;

#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    // generate server assets
    assets::load_assets(&ctx);

    info!("Total assets: {}", ctx.db.file().count());

    Ok(())
}

#[reducer(client_connected)]
pub fn identity_connected(_ctx: &ReducerContext) {

}

#[reducer(client_disconnected)]
pub fn identity_disconnected(_ctx: &ReducerContext) {

}
