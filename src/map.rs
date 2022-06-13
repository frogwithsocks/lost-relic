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
        app.add_startup_system(spawn_map)
            .add_system(add_colliders_to_map.after(spawn_map));
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
            translation: Vec3::new(BLOCK_SIZE * -12.0, BLOCK_SIZE * -6.0, 0.0),
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

fn add_colliders_to_map(mut commands: Commands, tile_query: Query<(Entity, &Tile, &TilePos)>) {
    for (tile_entity, tile, tile_pos) in tile_query.iter() {
        println!("{:?}", tile_pos);
        commands
            .entity(tile_entity)
            .insert(Transform::from_xyz(
                BLOCK_SIZE * tile_pos.0 as f32 + BLOCK_SIZE * -12.0 + BLOCK_SIZE / 2.0,
                BLOCK_SIZE * tile_pos.1 as f32 + BLOCK_SIZE * -6.0 + BLOCK_SIZE / 2.0,
                0.0,
            ))
            .insert(Collider {
                size: Vec2::new(BLOCK_SIZE, BLOCK_SIZE),
                r#type: ColliderType::Solid,
                on_ground: false,
            });
    }
}
