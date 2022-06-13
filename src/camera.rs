use bevy::{render::primitives::Frustum, prelude::*};
use crate::player::Player;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_camera);
    }
}

fn move_camera(mut camera_query: Query<&mut Transform, (With<Frustum>, Without<Player>)>, player_query: Query<&Transform, With<Player>>) {
    camera_query.single_mut().translation = player_query.single().translation;
}