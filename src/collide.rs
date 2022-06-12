use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

use crate::{velocity::Velocity, player::Player};

pub struct CollidePlugin;

impl Plugin for CollidePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system(check_collisions)
        .add_startup_system(test_floor);
    }
}

fn test_floor(mut commands: Commands) {
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::RED,
            custom_size: Some(Vec2::new(500f32, 100f32)),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0f32, -300f32, 0f32),
            ..default()
        },
        ..default()
    }).insert(Collider {
        size: Vec2::new(500f32, 100f32),
        r#type: ColliderType::Solid,
    });

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::RED,
            custom_size: Some(Vec2::new(500f32, 100f32)),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(-300f32, 150f32, 0f32),
            ..default()
        },
        ..default()
    }).insert(Collider {
        size: Vec2::new(500f32, 100f32),
        r#type: ColliderType::Solid,
    });
}

fn check_collision(colliders: &Vec<(&Transform, &Collider)>, size: Vec2, position: Vec3) -> Vec<Collision> {
    colliders.iter().filter_map(|(transform, collider)| collide(position, size, transform.translation, collider.size)).collect()
}

// TODO maybe sensors should contain a string which tells it which thing to switch on in the env
enum ColliderType {
    Solid,
    Sensor,
    Death,
}

#[derive(Component)]
pub struct Collider {
    size: Vec2,
    r#type: ColliderType,
}

fn check_collisions(
    mut commands: Commands,
    mut player_query: Query<(&mut Velocity, &mut Transform, &Sprite), With<Player>>,
    collider_query: Query<(Entity, &Transform, &Collider), Without<Player>>,
) {
    let (mut player_velocity, mut player_transform, player_sprite) = player_query.single_mut();
    for (collider_entity, transform, collider) in collider_query.iter() {
        if let Some(collision) = collide(transform.translation, collider.size, player_transform.translation, player_sprite.custom_size.unwrap()) {
            let pos = player_transform.translation;
            match collider.r#type {
                ColliderType::Solid => {
                    push(&collision, &mut player_transform, pos, player_sprite.custom_size.unwrap(), transform.translation, collider.size);
                    zero_velocity(&collision, &mut player_velocity);
                },
                ColliderType::Sensor => {
                    // Do stuff then return
                },
                ColliderType::Death => panic!("Player has died"),
            }
        }
    }
}

fn push(collision: &Collision, a_transform: &mut Transform, a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) {
    let push = match collision {
        Collision::Left =>   Vec2::new((b_pos.x + b_size.x/2f32) - (a_pos.x - a_size.x/2f32), 0f32),
        Collision::Right =>  Vec2::new((b_pos.x - b_size.x/2f32) - (a_pos.x + a_size.x/2f32), 0f32),
        Collision::Top =>    Vec2::new(0f32, (b_pos.y - b_size.y/2f32) - (a_pos.y + a_size.y/2f32)),
        Collision::Bottom => Vec2::new(0f32, (b_pos.y + b_size.y/2f32) - (a_pos.y - a_size.y/2f32)),
        Collision::Inside => Vec2::ZERO,
    };
    a_transform.translation += push.extend(0f32);
}

fn zero_velocity(collision: &Collision, velocity: &mut Velocity) {
    match collision {
        Collision::Left | Collision::Right => {
            velocity.linvel.x = 0f32;
        },
        Collision::Top | Collision::Bottom => {
            velocity.linvel.y = 0f32;
        },
        Collision::Inside => {
            velocity.linvel = Vec3::ZERO;
        },
    }
}