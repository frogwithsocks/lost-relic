#![feature(derive_default_enum)]
use animation::AnimationPlugin;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use camera::CameraPlugin;
use collide::{CollidePlugin, PlayerEvent};
use map::MapPlugin;
use player::PlayerPlugin;
use tiled_loader::TiledMapPlugin;
use ui::UiPlugin;
use velocity::VelocityPlugin;

mod animation;
mod camera;
mod collide;
mod map;
mod player;
mod tiled_loader;
mod triggers;
mod ui;
mod velocity;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<PlayerEvent>()
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_plugin(VelocityPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(CollidePlugin)
        .add_plugin(MapPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(UiPlugin)
        .add_event::<PlayerEvent>()
        .insert_resource(Msaa { samples: 1 })
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}
