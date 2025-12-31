mod assets;

use spacetimedb::{Identity, ReducerContext, ScheduleAt, Table, TimeDuration};

// Once second in micros
const DURATION: i64 = 1_000_000;
const TPS: i64 = 40;
const TICK: i64 = DURATION / TPS;

#[spacetimedb::table(name = player)]
pub struct Player {
    #[primary_key]
    identity: Identity,
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
    assets::init(ctx);

    assets::load_scripts(ctx);
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
    if !shared::is_initalized() {
        setup(ctx);
    }

    shared::tick_scripts().expect("Tick error");
}
