use spacetimedb::{Identity, ReducerContext, Table, Timestamp};

#[spacetimedb::table(name = player)]
pub struct Player {
    #[auto_inc]
    #[primary_key]
    id: u64,
}

#[spacetimedb::reducer(init)]
fn init(_ctx: &ReducerContext) {

}
