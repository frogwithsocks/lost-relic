use crate::player::Player;
use bevy::{prelude::*, render::primitives::Frustum};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_camera);
    }
}

#[derive(Component)]
pub struct CameraAnchor;

fn move_camera(
    mut camera_query: Query<
        &mut Transform,
        (
            With<Camera>,
            With<Frustum>,
            Without<Player>,
            Without<CameraAnchor>,
        ),
    >,
    camera_anchors: Query<&Transform, With<CameraAnchor>>,
    player_query: Query<&Transform, (With<Player>, Without<CameraAnchor>)>,
) {
    let player = match player_query.get_single() {
        Ok(p) => p,
        Err(_) => {
            camera_query.single_mut().translation = Vec3::Z * 999.9;
            return;
        }
    };
    let mut anchors = camera_anchors.iter();
    let mut camera = camera_query.single_mut();
    let mut closest = anchors.next().map(|v| v.translation).unwrap_or_default();
    for anchor in anchors {
        if anchor.translation.distance(player.translation) < closest.distance(player.translation) {
            closest = anchor.translation;
        }
    }
    if !camera.translation.is_finite() {
        camera.translation = Vec3::ZERO;
    }
    let mut midpoint: Vec3 = ((closest + camera.translation) / 2.0).round();
    midpoint.z = 999.9;
    camera.translation = midpoint;
}
