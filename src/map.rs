use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    collide::{Collider, ColliderKind},
    tiled_loader::{TiledMap, TiledMapBundle},
};

pub const BLOCK_SIZE: f32 = 96.0;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_map);
    }
}

#[derive(Component)]
pub struct CellTower;

pub fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<TiledMap> = asset_server.load("test.tmx");

    let map_entity = commands.spawn().id();
    commands.entity(map_entity).insert_bundle(TiledMapBundle {
        tiled_map: handle,
        map: Map::new(0u16, map_entity),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(3.0, 3.0, 1.0),
            ..default()
        },
        ..default()
    });

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::CRIMSON,
                custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(8.0 * BLOCK_SIZE, 5.0 * BLOCK_SIZE, 2.0),
            ..default()
        })
        .insert(Collider {
            size: Vec2::new(BLOCK_SIZE, BLOCK_SIZE),
            kind: ColliderKind::Death,
            ..default()
        });
}