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
