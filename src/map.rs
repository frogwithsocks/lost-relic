use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    collide::{Collider, ColliderKind},
    tiled_loader::{TiledMap, TiledMapBundle}, Level,
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

pub fn spawn_map(level: ResMut<Level>, mut commands: Commands, asset_server: Res<AssetServer>) {
    
    let handle: Handle<TiledMap> = asset_server.load(format!("{}.tmx", level.0).as_str());

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

}