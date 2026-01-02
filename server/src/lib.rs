mod assets;

use spacetimedb::{Identity, ReducerContext, ScheduleAt, Table, TimeDuration};

// Once second in micros
const DURATION: i64 = 1_000_000;
const TPS: i64 = 30;
const TICK: i64 = DURATION / TPS;

pub fn get_player(ctx: &ReducerContext) -> Option<Player> {
    ctx.db.player().identity().find(ctx.sender)
}

#[spacetimedb::table(name = player)]
pub struct Player {
    #[primary_key]
    identity: Identity,
    is_admin: bool,
}

#[spacetimedb::table(name=chunk)]
pub struct ChunkData {
    #[primary_key]
    /// Current position formated position key
    key: String,

    #[index(btree)]
    px: i32,
    #[index(btree)]
    py: i32,
    #[index(btree)]
    pz: i32,

    /// Raw chunk's data
    data: Vec<u8>
}

/// Setup core values and tables
fn setup(ctx: &ReducerContext) {
    shared::init();

    // Init assets (after Core initialization!)
    assets::init(ctx);
}

#[spacetimedb::reducer(init)]
fn init(ctx: &ReducerContext) {
    setup(ctx);

    let _ = ctx.db.ticker().try_insert(Ticker {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Interval(TimeDuration::from_micros(TICK))
    });
}

#[spacetimedb::table(name = ticker, scheduled(tick))]
struct Ticker {
    #[primary_key]
    scheduled_id: u64,
    scheduled_at: ScheduleAt
}

#[spacetimedb::reducer]
fn tick(ctx: &ReducerContext, _arg: Ticker) {
    if ctx.sender != ctx.identity() { return; }

    if !shared::is_initalized() {
        setup(ctx);
    }

    shared::tick_scripts().expect("Tick error");
}

#[spacetimedb::reducer]
/// Change asset or create new one
fn edit_asset(ctx: &ReducerContext, path: String, value: Vec<u8>) {
    // If player is exists and have admin rights
    let _is_admin = get_player(ctx)
        .and_then(|p| Some(p.is_admin))
        .unwrap_or(false);

    //Todo: If local-host (always admin access):
    //if is_admin {}

    // FIXME: private assets access 
    assets::add_raw_asset(ctx, path, value);
    return;
}

#[spacetimedb::reducer]
/// Change asset or create new one
fn remove_asset(ctx: &ReducerContext, path: String) {
    // If player is exists and have admin rights
    let _is_admin = get_player(ctx)
        .and_then(|p| Some(p.is_admin))
        .unwrap_or(false);

    //Todo: If local-host (always admin access):
    //if is_admin {}

    // FIXME: private assets access 
    assets::remove_raw_asset(ctx, path);
    return;
}
