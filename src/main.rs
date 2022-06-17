#![feature(derive_default_enum)]
use std::collections::HashMap;

use animation::AnimationPlugin;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use camera::CameraPlugin;
use collide::{CollidePlugin, GameEvent};
use map::MapPlugin;
use player::PlayerPlugin;
use tiled_loader::TiledMapPlugin;
use ui::UiPlugin;
use velocity::VelocityPlugin;
use event::EventPlugin;
use trigger::{ButtonRes, DoorRes};
use state::GameState;
use main_menu::MainMenuPlugin;
use slider::SliderPlugin;

mod animation;
mod camera;
mod collide;
mod map;
mod player;
mod tiled_loader;
mod trigger;
mod ui;
mod velocity;
mod event;
mod main_menu;
mod state;
mod slider;

pub struct Level(u32);
fn main() {
    App::new()
        .add_state(GameState::MainMenu)
        .add_plugins(DefaultPlugins)
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
        .insert_resource(ButtonRes(HashMap::new()))
        .insert_resource(DoorRes(HashMap::new())).insert_resource(Level(0))
        .insert_resource(Msaa { samples: 1 })
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}
