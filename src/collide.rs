use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

use crate::{map::BLOCK_SIZE, player::Player, velocity::Velocity};

pub struct CollidePlugin;

impl Plugin for CollidePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(check_collisions.after("velocity"))
            .add_startup_system(test_floor);
    }
}

fn test_floor(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(BLOCK_SIZE * 10.0, BLOCK_SIZE)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, -BLOCK_SIZE * 3.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Collider {
            size: Vec2::new(BLOCK_SIZE * 10.0, BLOCK_SIZE),
            r#type: ColliderType::Solid,
        });

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(BLOCK_SIZE * 10.0, BLOCK_SIZE)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-BLOCK_SIZE * 3.0, BLOCK_SIZE * 2.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Collider {
            size: Vec2::new(BLOCK_SIZE * 10.0, BLOCK_SIZE),
            r#type: ColliderType::Solid,
        });

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("floor_tile.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, -BLOCK_SIZE * 2.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(Collider {
            size: Vec2::new(BLOCK_SIZE, BLOCK_SIZE),
            r#type: ColliderType::Solid,
        });
}

pub enum PlayerEvent {
    Death,
    Button,
    Door,
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
    mut events: EventWriter<PlayerEvent>,
    mut player_query: Query<
        (
            &mut Velocity,
            &mut Transform,
            Option<&Sprite>,
            Option<&TextureAtlasSprite>,
            &mut Player,
        ),
        With<Player>,
    >,
    collider_query: Query<(Entity, &Transform, &Collider), Without<Player>>,
) {
    let (mut player_velocity, mut player_transform, maybe_sprite, maybe_atlas_sprite, mut player) =
        player_query.single_mut();
    let mut player_size: Option<Vec2> = None;
    player_size = maybe_sprite.map(|s| s.custom_size.unwrap());
    player_size = maybe_atlas_sprite.map(|s| s.custom_size.unwrap());

    let player_size = player_size.expect("Entity must have Sprite.custom_size or TextureAltasSprite.custom_size in order to preform collisions");

    player.on_ground = false;
    for (collider_entity, transform, collider) in collider_query.iter() {
        if let Some(collision) = collide(
            transform.translation,
            collider.size,
            player_transform.translation,
            player_size,
        ) {
            if matches!(collision, Collision::Bottom) {
                player.on_ground = true;
            }
            let pos = player_transform.translation;
            match collider.r#type {
                ColliderType::Solid => {
                    push(
                        &collision,
                        &mut player_transform,
                        pos,
                        player_size,
                        transform.translation,
                        collider.size,
                    );
                    zero_velocity(&collision, &mut player_velocity);
                }
                ColliderType::Sensor => {
                    events.send(PlayerEvent::Button);
                }
                ColliderType::Death => {
                    events.send(PlayerEvent::Death);
                }
            }
        }
    }
}

fn push(
    collision: &Collision,
    a_transform: &mut Transform,
    a_pos: Vec3,
    a_size: Vec2,
    b_pos: Vec3,
    b_size: Vec2,
) {
    let push = match collision {
        Collision::Left => Vec2::new((b_pos.x + b_size.x / 2.0) - (a_pos.x - a_size.x / 2.0), 0.0),
        Collision::Right => Vec2::new((b_pos.x - b_size.x / 2.0) - (a_pos.x + a_size.x / 2.0), 0.0),
        Collision::Top => Vec2::new(0.0, (b_pos.y - b_size.y / 2.0) - (a_pos.y + a_size.y / 2.0)),
        Collision::Bottom => {
            Vec2::new(0.0, (b_pos.y + b_size.y / 2.0) - (a_pos.y - a_size.y / 2.0))
        }
        Collision::Inside => Vec2::ZERO,
    };
    a_transform.translation += push.extend(0.0);
}

fn zero_velocity(collision: &Collision, velocity: &mut Velocity) {
    match collision {
        Collision::Left | Collision::Right => {
            velocity.linvel.x = 0.0;
        }
        Collision::Top | Collision::Bottom => {
            velocity.linvel.y = 0.0;
        }
        Collision::Inside => {
            velocity.linvel = Vec3::ZERO;
        }
    }
}
