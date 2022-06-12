use bevy::prelude::*;

pub const BLOCK_SIZE: f32 = 50.0;

#[derive(Component)]
pub struct CellTower;

fn spawn_cell_tower(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
    .spawn_bundle(SpriteBundle {
        texture: asset_server.load("cell_tower.png"),
        sprite: Sprite {
            custom_size: Some(Vec2::new(100.0, 200.0)),
            ..default()
        },
        transform: Transform {
            translation: Vec3::ZERO,
            ..default()
        },
        ..default()
    })
    .insert(CellTower);
}