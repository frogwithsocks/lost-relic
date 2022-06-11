use bevy::prelude::*;
use player::PlayerPlugin;
use velocity::VelocityPlugin;

mod player;
mod velocity;
mod triggers;
mod collide;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(VelocityPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}
