use bevy::{prelude::*, text::Text2dBounds};
use bevy_ecs_tilemap::prelude::*;

use crate::{
    state::GameState,
    tiled_loader::{TiledMap, TiledMapBundle, WorldObject},
    Level,
};

pub const BLOCK_SIZE: f32 = 96.0;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Play).with_system(spawn_map));
    }
}

#[derive(Component)]
pub struct ExitDoor;

#[derive(Component)]
pub struct CellTower;

pub fn spawn_map(level: ResMut<Level>, mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<TiledMap> = asset_server.load(format!("levels/{}.tmx", level.0).as_str());

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
    let mut spawn = |text: &str| {
        commands
            .spawn_bundle(Text2dBundle {
                text: Text::with_section(
                    text,
                    TextStyle {
                        font: asset_server.load("VT323-Regular.ttf"),
                        font_size: 50.0,
                        color: Color::BLACK,
                    },
                    TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                ),
                transform: Transform {
                    translation: Vec3::new(6.0 * BLOCK_SIZE, 5.5 * BLOCK_SIZE, 5.0),
                    ..default()
                },
                text_2d_bounds: Text2dBounds {
                    // Wrap text in the rectangle
                    size: Size {
                        width: BLOCK_SIZE * 6.0,
                        height: BLOCK_SIZE * 4.0,
                    },
                },
                ..default()
            })
            .insert(WorldObject);
    };

    match level.0 {
        0 => spawn("WASD to move"),
        1 => spawn("Latency will delay your inputs.\nMove Carefully."),
        4 => spawn("Press R to restart the level."),
        8 => spawn("You Win!\nThanks for playing"),
        _ => (),
    };
    //TODO add instructions to world
}
