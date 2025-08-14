// todo: chunks, blocks metadata, client-side rendering,
// server-side mesher, and meshes cache

use log::debug;
use bevy_tasks::*;
use spacetimedb::{
    table, Table, reducer,
    ReducerContext, ScheduleAt, TimeDuration
};

pub mod math;
mod player;
mod assets;
mod chunks;
mod mesher;

use assets::asset;
use math::*;

#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    // generate server assets
    assets::load_assets(&ctx);

    debug!("Total assets: {}", ctx.db.asset().count());

    // create blocks and chunks data
    chunks::init_blocks(&ctx);
    mesher::init_mesher();
    chunks::generate_world(&ctx);

    AsyncComputeTaskPool::get_or_init(|| TaskPool::new());

    // Begin ticks loop
    let delta = TimeDuration::from_micros(10_000);
    ctx.db.tick_schedule().insert(TickSchedule {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Interval(delta),
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

#[table(name = tick_schedule, scheduled(tick))]
pub struct TickSchedule {
    #[primary_key]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
    
    pub tick: u128
}

#[reducer]
fn tick(ctx: &ReducerContext, mut arg: TickSchedule) -> Result<(), String> {
    if ctx.sender != ctx.identity() {
        return Err("Tick may not be invoked by clients, only via scheduling.".into());
    }

    // Begin tick
    arg.tick += 1;

    // Run mesher tasks
    mesher::proceed_mesher(ctx);

    // Process tasks
    AsyncComputeTaskPool::get()
        .with_local_executor(|executor| { executor.try_tick() });

    ctx.db.tick_schedule().scheduled_id().update(arg);
    Ok(())
}
