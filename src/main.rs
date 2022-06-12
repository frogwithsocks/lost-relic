use bevy::prelude::*;
use collide::{CollidePlugin, PlayerEvent};
use player::PlayerPlugin;
use velocity::VelocityPlugin;
use map::MapPlugin;
use animation::AnimationPlugin;

mod collide;
mod player;
mod triggers;
mod velocity;
mod map;
mod animation;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<PlayerEvent>()
        .insert_resource(Msaa { samples: 1 })
        .add_plugin(VelocityPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(CollidePlugin)
        .add_plugin(MapPlugin)
        .add_plugin(AnimationPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}
