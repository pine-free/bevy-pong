use bevy::ecs::component::Component;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, Component)]
pub struct PaddleSpeed(pub f32);

#[derive(Clone, Copy, Debug, Default, Component)]
pub struct BallSpeed(pub f32);

#[derive(Clone, Copy, Debug, Default, Component)]
/// Width and height of the arena
pub struct ArenaSize(pub Vec2);

#[derive(Clone, Copy, Debug, Default, Component)]
pub struct GameConfig;

pub fn game_config() -> impl Scene {
    bsn! {
        #Config
        GameConfig
        PaddleSpeed(10.)
        ArenaSize(vec2(450., 300.))
    }
}
