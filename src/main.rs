use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    game_config::ArenaSize,
    input::{Action, InputConfig},
    platform::{Platform, Player1, Player2},
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
    use bevy::prelude::*;

    #[derive(Clone, Copy, Debug, Component, Default)]
    #[require(Transform)]
    pub struct Ball;

    #[derive(Clone, Copy, Debug, Component, Default)]
    pub struct BallVelocity(Vec2);

    pub fn ball(radius: f32) -> impl Scene {
        bsn! {
            #Ball
            Ball
            Mesh2d(asset_value(Circle::new(radius)))
            MeshMaterial2d<ColorMaterial>(asset_value(Color::WHITE))
            BallVelocity(Vec2::splat(10.))
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
    pub struct Platform;

    #[derive(Clone, Copy, Debug, Component, Default)]
    pub struct Player1;

    #[derive(Clone, Copy, Debug, Component, Default)]
    pub struct Player2;

    #[derive(Clone, Copy, Debug, Component, Default)]
    pub struct PlatformDimensions(pub Vec2);

    pub fn platform(position: Vec2, dimensions: Vec2) -> impl Scene {
        bsn! {
            Platform
            Transform::from_translation(position.extend(0.0))
            PlatformDimensions(dimensions)
            Mesh2d(asset_value(Rectangle::from_size(dimensions)))
            MeshMaterial2d<ColorMaterial>(asset_value(ColorMaterial::from_color(Color::WHITE)))
        }
    }

    pub fn player_input(
        action: On<PlayerAction>,
        mut position: Populated<&mut Transform, With<Platform>>,
        speed: Single<&PaddleSpeed>,
        dimensions: Populated<&PlatformDimensions>,
        arena_size: Option<Single<&ArenaSize>>,
    ) {
        let mut self_pos = position
            .get_mut(action.event_target())
            .expect("should always be a platform");
        let PaddleSpeed(sp) = **speed;
        let PlatformDimensions(Vec2 { y: pl_height, .. }) = dimensions
            .get(action.event_target())
            .expect("must have dimensions");

        let mut next_translation = self_pos.translation.clone();
        match action.event().action {
            Action::P1MoveLeft | Action::P2MoveLeft => next_translation.y += sp,
            Action::P1MoveRight | Action::P2MoveRight => next_translation.y -= sp,
        }

        if arena_size.is_some()
            && let ArenaSize(Vec2 { y: arena_h, .. }) = **arena_size.unwrap()
            && next_translation.y.abs() + pl_height / 2. > arena_h
        {
            return;
        }

        self_pos.translation = next_translation;
    }
}

mod controls {
    use bevy::ecs::{entity::Entity, query::With, system::Single};
    use bevy::prelude::*;

    use crate::game_config::ArenaSize;
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
        #Platform1
        Player1
        @platform::platform(vec2(-400., 0.), vec2(20., 200.))
        on(platform::player_input)
        --
        #Platform2
        Player2
        @platform::platform(vec2(400., 0.), vec2(20., 200.))
        on(platform::player_input)
        --
        #Arena
        @arena::arena(**arena_size)

    });
    info_once!("scene spawned!");
}

fn test_q(platforms: Query<Entity, With<Platform>>) {
    for p in platforms {
        info_once!("got platform {p:?}");
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, input::plugin, controls::plugin))
        .add_systems(Startup, (game_config::game_config.spawn(), setup).chain())
        .add_systems(Update, test_q)
        .insert_resource(InputConfig::from(HashMap::from([
            (KeyCode::KeyA, Action::P1MoveLeft),
            (KeyCode::KeyD, Action::P1MoveRight),
            (KeyCode::KeyJ, Action::P2MoveLeft),
            (KeyCode::KeyL, Action::P2MoveRight),
        ])))
        .run();
}
