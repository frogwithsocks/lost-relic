use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};
use bevy_ecs_tilemap::{Map, MapQuery, Tile, TilePos};

use crate::{
    collide::{Collider, ColliderType},
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
pub struct CellTower {
    pub offset: Vec3,
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<TiledMap> = asset_server.load("test.tmx");

    let map_entity = commands.spawn().id();

    commands.entity(map_entity).insert_bundle(TiledMapBundle {
        tiled_map: handle,
        map: Map::new(0u16, map_entity),
        transform: Transform {
            translation: Vec3::new(BLOCK_SIZE * -15.0, BLOCK_SIZE * -7.0, 0.0),
            scale: Vec3::new(3.0, 3.0, 1.0),
            ..default()
        },
        ..default()
    });

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("cell_tower.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE * 2.0)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::ZERO,
                ..default()
            },
            ..default()
        })
        .insert(CellTower {
            offset: Vec3::new(0.0, BLOCK_SIZE / 2.0, 0.0),
        });
}
