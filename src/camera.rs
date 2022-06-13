use crate::player::Player;
use bevy::{prelude::*, render::primitives::Frustum};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_camera.after("player"));
    }
}

#[derive(Component)]
pub struct CameraAnchor;

fn move_camera(
    mut camera_query: Query<&mut Transform, (With<Camera>, With<Frustum>, Without<Player>, Without<CameraAnchor>)>,
    camera_anchors: Query<&Transform, With<CameraAnchor>>,
    player_query: Query<&Transform, (With<Player>, Without<CameraAnchor>)>,
) {
    let player = player_query.single();
    let mut closest = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    for anchor in camera_anchors.iter() {
        if anchor.translation.distance(player.translation) < closest.distance(player.translation) {
            closest = anchor.translation;
        }
    }
    println!("{:?}", camera_query.single_mut().translation);
    closest.z = 999.9;
    camera_query.single_mut().translation = closest;
}
