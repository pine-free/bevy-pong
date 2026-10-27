use bevy::prelude::*;

use crate::{
    game_config::{ArenaSize, PaddleSpeed},
    input::{Action, PlayerAction},
};

pub fn plugin(app: &mut App) {
    app.add_observer(increase_player_score);
}

#[derive(Clone, Copy, Debug, Component, Default)]
#[require(Transform)]
pub struct Paddle;

#[derive(Clone, Copy, Debug, Component, Default)]
pub struct PlayerScore(u32);

#[derive(Clone, Copy, Debug, Event)]
pub enum IncreasePlayerScore {
    Player1,
    Player2,
}

#[derive(Clone, Copy, Debug, Component, Default)]
pub struct Player1;

#[derive(Clone, Copy, Debug, Component, Default)]
pub struct Player2;

#[derive(Clone, Copy, Debug, Component, Default)]
pub struct PaddleDimensions(pub Vec2);

pub fn platform(position: Vec2, dimensions: Vec2) -> impl Scene {
    let dim = dimensions * 2.;
    bsn! {
        Paddle
        Transform::from_translation(position.extend(0.0))
        PaddleDimensions(dim)
        Mesh2d(asset_value(Rectangle::from_size(dimensions)))
        MeshMaterial2d<ColorMaterial>(asset_value(ColorMaterial::from_color(Color::WHITE)))
        PlayerScore
    }
}

pub fn player_input(
    action: On<PlayerAction>,
    mut position: Populated<&mut Transform, With<Paddle>>,
    speed: Single<&PaddleSpeed>,
    dimensions: Populated<&PaddleDimensions>,
    arena_size: Option<Single<&ArenaSize>>,
) {
    let mut self_pos = position
        .get_mut(action.event_target())
        .expect("should always be a platform");
    let PaddleSpeed(sp) = **speed;
    let PaddleDimensions(Vec2 { y: pl_height, .. }) = dimensions
        .get(action.event_target())
        .expect("must have dimensions");

    let mut next_translation = self_pos.translation.clone();
    match action.event().action {
        Action::P1MoveLeft | Action::P2MoveLeft => next_translation.y += sp,
        Action::P1MoveRight | Action::P2MoveRight => next_translation.y -= sp,
    }

    if arena_size.is_some()
        && let ArenaSize(Vec2 { y: arena_h, .. }) = **arena_size.unwrap()
        && next_translation.y.abs() + pl_height / 4. > arena_h
    {
        return;
    }

    self_pos.translation = next_translation;
}

pub fn increase_player_score(
    event: On<IncreasePlayerScore>,
    mut player1: Single<&mut PlayerScore, With<Player1>>,
    mut player2: Single<&mut PlayerScore, (With<Player2>, Without<Player1>)>,
) {
    match event.event() {
        IncreasePlayerScore::Player1 => player1.0 += 1,
        IncreasePlayerScore::Player2 => player2.0 += 1,
    }

    info!("score: {} {}", player1.0, player2.0);
}

pub fn draw_paddle_gizmos(mut gizmos: Gizmos, paddles: Populated<(&Transform, &PaddleDimensions)>) {
    for (p_tr, p_dim) in paddles {
        gizmos.rect_2d(
            Isometry2d::from_translation(p_tr.translation.xy()),
            p_dim.0 / 2.,
            LinearRgba::RED,
        );
    }
}
