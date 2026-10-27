use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    ball::BallVelocity,
    game_config::ArenaSize,
    input::{Action, InputConfig},
    platform::{Player1, Player2, draw_paddle_gizmos},
};

mod game_config {
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
}

mod arena {
    use crate::game_config::ArenaSize;
    use bevy::prelude::*;

    pub fn arena(size: ArenaSize) -> impl Scene {
        let ArenaSize(Vec2 { x, y }) = size;
        bsn! {
            #Arena
            Mesh2d(asset_value(Polyline2d::new(vec![
                (x, y),
                (x, -y),
                (-x, -y),
                (-x, y),
                (x, y)
            ].into_iter().map(|(x, y)| vec2(x, y)))))
            MeshMaterial2d<ColorMaterial>(asset_value(Color::WHITE))
        }
    }
}

mod ball {
    use bevy::{ecs::query::QueryData, prelude::*};

    use crate::{game_config::ArenaSize, platform::PaddleDimensions};

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

                let next_ball_pos = ball_pos.xy() + vel.0.xy();
                if wall_up.contains(next_ball_pos) || wall_down.contains(next_ball_pos) {
                    vel.0.y *= -1.;
                }

                if wall_left.contains(next_ball_pos) || wall_right.contains(next_ball_pos) {
                    vel.0.x *= -1.;
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
}

mod platform {
    use bevy::prelude::*;

    use crate::{
        game_config::{ArenaSize, PaddleSpeed},
        input::{Action, PlayerAction},
    };

    #[derive(Clone, Copy, Debug, Component, Default)]
    #[require(Transform)]
    pub struct Paddle;

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

    pub fn draw_paddle_gizmos(
        mut gizmos: Gizmos,
        paddles: Populated<(&Transform, &PaddleDimensions)>,
    ) {
        for (p_tr, p_dim) in paddles {
            gizmos.rect_2d(
                Isometry2d::from_translation(p_tr.translation.xy()),
                p_dim.0 / 2.,
                LinearRgba::RED,
            );
        }
    }
}

mod controls {
    use bevy::ecs::{entity::Entity, query::With, system::Single};
    use bevy::prelude::*;

    use crate::input::PlayerAction;
    use crate::platform::Player2;
    use crate::{input::TriggeredAction, platform::Player1};

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
}

mod input {
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
}

#[derive(Clone, Copy, Debug, Component, Default)]
struct MainCamera;

fn setup(mut commands: Commands, arena_size: Single<&ArenaSize>) {
    commands.spawn_scene_list(bsn_list! {
        Camera2d
        MainCamera
        --
        #Player1
        Player1
        @platform::platform(vec2(-400., 0.), vec2(20., 200.))
        on(platform::player_input)
        --
        #Player2
        Player2
        @platform::platform(vec2(400., 0.), vec2(20., 200.))
        on(platform::player_input)
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
