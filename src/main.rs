use animation::AnimationPlugin;
use bevy::prelude::*;
use collide::{CollidePlugin, PlayerEvent};
use map::MapPlugin;
use player::PlayerPlugin;
use std::collections::HashMap;
use velocity::VelocityPlugin;

mod animation;
mod collide;
mod map;
mod player;
mod triggers;
mod velocity;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(VelocityPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(CollidePlugin)
        .add_plugin(MapPlugin)
        .add_plugin(AnimationPlugin)
        .add_event::<PlayerEvent>()
        .insert_resource(Msaa { samples: 1 })
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}
