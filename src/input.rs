use bevy::prelude::*;
use std::collections::HashMap;

pub fn plugin(app: &mut App) {
    app.init_resource::<InputConfig>()
        .add_systems(FixedUpdate, input_propagation);
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    P1MoveLeft,
    P1MoveRight,
    P2MoveLeft,
    P2MoveRight,
}

#[derive(Clone, Copy, Debug, EntityEvent)]
pub struct PlayerAction {
    entity: Entity,
    pub action: Action,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct TriggeredAction(pub Action);

impl PlayerAction {
    pub fn new(entity: Entity, action: Action) -> Self {
        Self { entity, action }
    }
}

#[derive(Clone, Debug, Resource, Default)]
pub struct InputConfig {
    mapping: HashMap<KeyCode, Action>,
}

impl From<HashMap<KeyCode, Action>> for InputConfig {
    fn from(value: HashMap<KeyCode, Action>) -> Self {
        Self { mapping: value }
    }
}

fn input_propagation(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    input_cfg: Res<InputConfig>,
) {
    for (k, a) in input_cfg.mapping.iter() {
        if input.pressed(*k) {
            commands.trigger(TriggeredAction(*a));
        }
    }
}
