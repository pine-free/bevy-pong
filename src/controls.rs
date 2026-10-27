use bevy::ecs::{entity::Entity, query::With, system::Single};
use bevy::prelude::*;

use crate::input::PlayerAction;
use crate::player::Player2;
use crate::{input::TriggeredAction, player::Player1};

pub fn plugin(app: &mut App) {
    app.add_observer(control_logic);
}

fn control_logic(
    action: On<TriggeredAction>,
    mut commands: Commands,
    player1: Single<Entity, With<Player1>>,
    player2: Single<Entity, With<Player2>>,
) {
    let TriggeredAction(a) = action.event();
    match a {
        crate::input::Action::P1MoveLeft | crate::input::Action::P1MoveRight => commands
            .entity(*player1)
            .trigger(|e| PlayerAction::new(e, *a)),
        crate::input::Action::P2MoveLeft | crate::input::Action::P2MoveRight => commands
            .entity(*player2)
            .trigger(|e| PlayerAction::new(e, *a)),
    };
}
