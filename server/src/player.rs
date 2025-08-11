use super::math::*;
use spacetimedb::{
    reducer, table, Table,
    Identity, ReducerContext,
    client_visibility_filter, Filter
};

#[client_visibility_filter]
const SELF_FILTER: Filter = Filter::Sql(
    "SELECT * FROM player WHERE player.identity = :sender"
);

#[client_visibility_filter]
const ONLINE_FILTER: Filter = Filter::Sql(
    "SELECT * FROM player WHERE player.online"
);

#[table(name = player, public)]
pub struct Player {
    #[auto_inc]
    #[primary_key]
    id: u64,
    name: String,
    #[unique]
    identity: Identity,
    position: StVec3,
    online: bool
}

#[table(name = scanner)]
// todo: destroy if player is offline
pub struct Scanner {
    #[primary_key]
    // linked with player identity
    identity: Identity,
    chunk: StIVec3
}

#[reducer]
pub fn create_player(ctx: &ReducerContext, name: String) -> Result<(), String> {
    if ctx.db.player().identity().find(ctx.sender).is_some() {
        return Err("Player is already exists!".to_string());
    }

    ctx.db.player().insert(Player {
        id: 0,
        name,
        identity: ctx.sender,
        position: vec3(0.0, 40.0, 0.0).into(),
        online: false
    });

    Ok(())
}


#[reducer]
pub fn join(ctx: &ReducerContext) -> Result<(), String> {
    let Some(mut player) = ctx.db.player().identity().find(ctx.sender) else {
        return Err("Player is not exists!".to_string());
    };

    if !player.online {
        return Err("Player is already joined".to_string())
    }

    player.online = true;
    ctx.db.player().identity().update(player);

    Ok(())
}
