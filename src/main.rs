use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    ball::BallVelocity,
    game_config::ArenaSize,
    input::{Action, InputConfig},
    player::{Player1, Player2, draw_paddle_gizmos},
};

mod game_config;

mod arena;

mod ball;

mod player;

mod controls;

mod input;

#[derive(Clone, Copy, Debug, Component, Default)]
struct MainCamera;

fn setup(mut commands: Commands, arena_size: Single<&ArenaSize>) {
    commands.spawn_scene_list(bsn_list! {
        Camera2d
        MainCamera
        --
        #Player1
        Player1
        @player::platform(vec2(-400., 0.), vec2(20., 200.))
        on(player::player_input)
        --
        #Player2
        Player2
        @player::platform(vec2(400., 0.), vec2(20., 200.))
        on(player::player_input)
        --
        #Arena
        @arena::arena(**arena_size)
        --
        #Ball
        @ball::ball(10.)
        BallVelocity(vec2(5., 5.))

    });
    info_once!("scene spawned!");
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            input::plugin,
            controls::plugin,
            ball::plugin,
        ))
        .add_systems(Startup, (game_config::game_config.spawn(), setup).chain())
        // .add_systems(Update, draw_paddle_gizmos)
        .insert_resource(InputConfig::from(HashMap::from([
            (KeyCode::KeyA, Action::P1MoveLeft),
            (KeyCode::KeyD, Action::P1MoveRight),
            (KeyCode::KeyJ, Action::P2MoveLeft),
            (KeyCode::KeyL, Action::P2MoveRight),
        ])))
        .run();
}
