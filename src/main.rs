#![feature(derive_default_enum)]
use std::collections::HashMap;

use animation::AnimationPlugin;
use bevy::{prelude::*, window::WindowMode};
use bevy_ecs_tilemap::TilemapPlugin;
use camera::CameraPlugin;
use collide::{CollidePlugin, GameEvent};
use event::EventPlugin;
use main_menu::MainMenuPlugin;
use map::MapPlugin;
use player::PlayerPlugin;
use slider::SliderPlugin;
use state::GameState;
use tiled_loader::TiledMapPlugin;
use trigger::DoorRes;
use ui::UiPlugin;
use velocity::VelocityPlugin;

mod animation;
mod camera;
mod collide;
mod event;
mod main_menu;
mod map;
mod player;
mod slider;
mod state;
mod tiled_loader;
mod trigger;
mod ui;
mod velocity;

pub struct Level(u32);
fn main() {
    App::new()
        .add_state(GameState::MainMenu)
        .add_plugins(DefaultPlugins)
        .insert_resource(WindowDescriptor {
            title: String::from("Connection Severed"),
            resizable: false,
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        })
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_event::<GameEvent>()
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_plugin(VelocityPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(CollidePlugin)
        .add_plugin(MapPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(EventPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(SliderPlugin)
        .insert_resource(DoorRes(HashMap::new()))
        .insert_resource(Level(0))
        .insert_resource(Msaa { samples: 1 })
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}
