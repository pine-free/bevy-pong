use bevy::{ecs::query::QueryData, prelude::*};

use crate::{
    game_config::ArenaSize,
    player::{IncreasePlayerScore, PaddleDimensions},
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (detect_ball_wallbounce, detect_ball_paddlebounce, move_ball).chain(),
    )
    .add_observer(bounce_ball);
}

#[derive(Clone, Copy, Debug, Component, Default)]
#[require(Transform)]
pub struct Ball;

#[derive(Clone, Copy, Debug, Component, Default)]
pub struct BallVelocity(pub Vec2);

impl From<Vec2> for BallVelocity {
    fn from(value: Vec2) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Component, Default)]
pub struct BallRadius(pub f32);

impl From<f32> for BallRadius {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Event)]
pub enum BallBounce {
    FromWall,
    FromPaddle,
}

#[derive(QueryData)]
#[query_data(derive(Debug))]
pub struct BallQuery {
    entity: Entity,
    transform: &'static Transform,
    velocity: &'static BallVelocity,
    radius: &'static BallRadius,
}

pub fn ball(radius: f32) -> impl Scene {
    bsn! {
        #Ball
        Ball
        Mesh2d(asset_value(Circle::new(radius)))
        MeshMaterial2d<ColorMaterial>(asset_value(Color::WHITE))
        BallVelocity(Vec2::splat(10.))
        BallRadius(10.)
    }
}

fn move_ball(mut ball: Single<(&mut Transform, &BallVelocity)>) {
    let (tr, vel) = &mut *ball;
    let BallVelocity(velocity) = &**vel;
    tr.translation += velocity.extend(0.);
}

fn bounce_ball(
    event: On<BallBounce>,
    mut commands: Commands,
    mut ball: Single<(&mut BallVelocity, &BallRadius, &Transform)>,
    arena: Single<&ArenaSize>,
) {
    info_once!("ball bounce detected!");
    let (
        vel,
        BallRadius(ball_r),
        Transform {
            translation: ball_pos,
            ..
        },
    ) = &mut *ball;
    let ArenaSize(Vec2 {
        x: arena_w,
        y: arena_h,
    }) = *arena;
    match event.event() {
        BallBounce::FromWall => {
            let wall_up =
                Rect::from_corners(vec2(*arena_w, *arena_h), vec2(-arena_w, arena_h - ball_r));
            let wall_down =
                Rect::from_corners(vec2(*arena_w, -arena_h), vec2(-arena_w, -arena_h + ball_r));
            let wall_right =
                Rect::from_corners(vec2(*arena_w, *arena_h), vec2(arena_w - ball_r, -arena_h));
            let wall_left =
                Rect::from_corners(vec2(-arena_w, *arena_h), vec2(-arena_w + ball_r, -arena_h));

            let walls = vec![wall_up, wall_down, wall_left, wall_right];
            let next_ball_pos = ball_pos.xy() + vel.0.xy();
            let hit_wall = {
                let v = walls
                    .iter()
                    .filter(|w| w.contains(next_ball_pos))
                    .take(1)
                    .collect::<Vec<_>>();
                v.first().cloned().expect("definitely hit some wall")
            };

            if hit_wall == &wall_up || hit_wall == &wall_down {
                vel.0.y *= -1.;
            } else if hit_wall == &wall_left || hit_wall == &wall_right {
                vel.0.x *= -1.;
                if hit_wall == &wall_left {
                    commands.trigger(IncreasePlayerScore::Player2);
                } else if hit_wall == &wall_right {
                    commands.trigger(IncreasePlayerScore::Player1);
                }
            }
        }
        BallBounce::FromPaddle => vel.0.x *= -1.0,
    };
}

fn detect_ball_wallbounce(
    ball: Single<BallQuery>,
    arena: Single<&ArenaSize>,
    mut commands: Commands,
) {
    let ArenaSize(arena_size) = *arena;

    // Bouncing from walls
    if !Rect::from_center_half_size(Vec2::ZERO, arena_size - Vec2::splat(ball.radius.0))
        .contains(ball.transform.translation.xy() + ball.velocity.0)
    {
        info_once!("ball bounced!");
        commands.trigger(BallBounce::FromWall);
    }
}

fn detect_ball_paddlebounce(
    ball: Single<BallQuery>,
    mut gizmos: Gizmos,
    mut commands: Commands,
    paddles: Populated<(&Transform, &PaddleDimensions)>,
) {
    for (paddle_tr, PaddleDimensions(paddle_dim)) in paddles {
        let paddle_rect = Rect::from_center_size(Vec2::ZERO, *paddle_dim / 2.)
            .translate(paddle_tr.translation.xy());
        // gizmos.rect_2d(
        //     Isometry2d::from_translation(paddle_tr.translation.xy()),
        //     *paddle_dim / 2.,
        //     LinearRgba::GREEN,
        // );
        if paddle_rect.contains(
            ball.transform.translation.xy()
                + ball.velocity.0
                + ball.velocity.0.normalize() * ball.radius.0,
        ) {
            info!("paddlebounce!");
            commands.trigger(BallBounce::FromPaddle);
        }
    }
}
