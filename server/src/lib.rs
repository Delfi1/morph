// todo: chunks, blocks metadata, client-side rendering,
// server-side mesher, and meshes cache

use log::debug;
use bevy_tasks::*;
use spacetimedb::{
    reducer, table, ReducerContext, ScheduleAt, Table, TimeDuration, Timestamp
};

pub mod math;
mod player;
mod assets;
mod chunks;
mod mesher;

use assets::asset;
use math::*;

// Tick time in micros
pub const TICK: i64 = 50_000;

#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    // generate server assets
    assets::load_assets(&ctx);
        let img = assets::load_asset_image(ctx, "perlin_32.png")
        .expect("Нет ассета perlin_32.png");

    debug!("Total assets: {}", ctx.db.asset().count());

    // create blocks and chunks data
    chunks::init_blocks(&ctx);
    mesher::init_mesher();
    chunks::generate_world(&ctx);

    AsyncComputeTaskPool::get_or_init(|| TaskPool::new());

    // Begin ticks loop
    let delta = TimeDuration::from_micros(TICK);
    ctx.db.ticks().insert(Ticks {
        id: 0,
        scheduled_at: ScheduleAt::Interval(delta),
        previous: ctx.timestamp,
        tickrate: 0.0,
        tick: 0
    });

    Ok(())
}

#[reducer(client_connected)]
pub fn identity_connected(_ctx: &ReducerContext) {

}

#[reducer(client_disconnected)]
pub fn identity_disconnected(_ctx: &ReducerContext) {

}

#[table(name = ticks, scheduled(run_tick), public)]
pub struct Ticks {
    #[primary_key]
    pub id: u64,
    pub scheduled_at: ScheduleAt,

    previous: Timestamp,
    pub tickrate: f64,
    pub tick: u128
}

#[reducer]
fn run_tick(ctx: &ReducerContext, mut arg: Ticks) -> Result<(), String> {
    if ctx.sender != ctx.identity() {
        return Err("Tick may not be invoked by clients, only via scheduling.".into());
    }

    // Begin tick
    arg.tick += 1;
    let delta = ctx.timestamp.duration_since(arg.previous).unwrap();
    arg.tickrate = 1.0 / delta.as_secs_f64();
    arg.previous = ctx.timestamp;

    // Run mesher tasks
    mesher::proceed_mesher(ctx);

    // Process tasks
    AsyncComputeTaskPool::get()
        .with_local_executor(|executor| { executor.try_tick() });

    ctx.db.ticks().id().update(arg);
    Ok(())
}
