use bevy::prelude::*;

use crate::{velocity::Velocity, player::Player};

struct CollidePlugin;

impl Plugin for CollidePlugin {
    fn build(&self, app: &mut App) {
        // app.add_system()
    }
}
// TODO maybe sensors should contain a string which tells it which thing to switch on in the env
enum ColliderType {
    Solid,
    Sensor,
    Death,
}

#[derive(Component)]
struct Collider {
    size: Vec2,
    r#type: ColliderType,
}

fn check_collisions(
    mut commands: Commands,
    mut player_query: Query<(&mut Velocity, &Transform, &Sprite), With<Player>>,
    collider_query: Query<(Entity, &Transform, &Collider)>,
) {
    let (mut player_velocity, player_transform, player_sprite) = player_query.single_mut();
    for (collider_entity, transform, collider) in collider_query.iter() {
        match collider.r#type {
            ColliderType::Solid => (),
            ColliderType::Sensor => {
                // Do stuff then return
            },
            ColliderType::Death => panic!("Player has died"),
        }
    }
}
