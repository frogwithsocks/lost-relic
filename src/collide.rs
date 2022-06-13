use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use std::{collections::{HashMap, HashSet}};

use crate::{
    map::BLOCK_SIZE,
    player::Player,
    velocity::{Gravity, Velocity},
};

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
            on_ground: false,
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
            r#type: ColliderType::Movable,
            on_ground: false,
        })
        .insert(Velocity::default())
        .insert(Gravity::default());
}

pub enum PlayerEvent {
    Solid(u32),
    Sensor(u32),
    Death,
}

// TODO maybe sensors should contain a string which tells it which thing to switch on in the env
#[derive(Debug, Clone, Default)]
pub enum ColliderType {
    #[default]
    Solid,
    Sensor,
    Movable,
    Death,
}

#[derive(Component, Clone, Default)]
pub struct Collider {
    pub size: Vec2,
    pub r#type: ColliderType,
    pub on_ground: bool,
}

/*fn check_collisions(
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
    mut collider_query: Query<(Entity, &mut Transform, &Collider), Without<Player>>,
) {
    let (mut player_velocity, mut player_transform, maybe_sprite, maybe_atlas_sprite, mut player) =
        player_query.single_mut();
    let mut player_size: Option<Vec2> = None;
    player_size = maybe_sprite.map(|s| s.custom_size.unwrap());
    player_size = maybe_atlas_sprite.map(|s| s.custom_size.unwrap());

    let player_size = player_size.expect("Entity must have Sprite.custom_size or TextureAltasSprite.custom_size in order to preform collisions");
    let mut second_update: Vec<(Entity, Vec3)> = Vec::new();
    player.on_ground = false;
    for (collider_entity, collider_transform, collider) in collider_query.iter() {
        if let Some(collision) = collide(
            collider_transform.translation,
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
                    player_transform.translation += push_force(&collision, pos, player_size, collider_transform.translation, collider.size);
                    zero_velocity(&collision, &mut player_velocity);
                    events.send(PlayerEvent::Solid(collider_entity.id()));
                }
                ColliderType::Sensor => {
                    events.send(PlayerEvent::Sensor(collider_entity.id()));
                }
                ColliderType::Death => {
                    events.send(PlayerEvent::Death);
                }
                ColliderType::Movable => {
                    let pushed_pos = collider_transform.translation
                        - push_force(
                            &collision,
                            pos,
                            player_size,
                            collider_transform.translation,
                            collider.size,
                        );
                    if match collision {
                        Collision::Left | Collision::Right => true,
                        _ => false,
                    } && !collider_query
                        .iter()
                        .filter(|(e, _, _)| e.id() != collider_entity.id())
                        .any(|(_, other_transform, other_collider)| {
                            collide(
                                other_transform.translation,
                                other_collider.size,
                                pushed_pos,
                                collider.size,
                            )
                            .is_some()
                        })
                    {
                        second_update.push((collider_entity, pushed_pos));
                    } else {
                        player_transform.translation += push_force(
                            &collision,
                            pos,
                            player_size,
                            collider_transform.translation,
                            collider.size,
                        );
                        zero_velocity(&collision, &mut player_velocity);
                    }
                }
            }
        }
    }

    for (entity, new_pos) in second_update {
        let mut collider_transform = collider_query
            .get_component_mut::<Transform>(entity)
            .unwrap();
        collider_transform.translation = new_pos;
    }
}*/

fn check_collisions(
    mut events: EventWriter<PlayerEvent>,
    mut collider_query: Query<(Entity, &mut Transform, Option<&mut Velocity>, &mut Collider)>,
) {
    let mut position_update: HashMap<Entity, Vec3> = HashMap::new();
    let mut velocity_update: Vec<(Entity, Vec3)> = Vec::new();
    let mut grounded_update: Vec<Entity> = Vec::new();
    for (entity, transform, velocity_opt, collider) in
        collider_query.iter().filter(|(_, _, v, _)| v.is_some())
    {
        for (other_entity, other_transform, _, other_collider) in
            collider_query.iter().filter(|(e, _, _, c)| {
                e.id() != entity.id() && matches!(c.r#type, ColliderType::Movable)
            })
        {
            let other_pos = other_transform.translation;
            let pos = transform.translation;
            if let Some(collision) = collide(other_pos, other_collider.size, pos, collider.size) {
                if matches!(collision, Collision::Bottom) {
                    grounded_update.push(entity);
                }
                let push = push_force(
                    &collision,
                    pos,
                    collider.size,
                    other_pos,
                    other_collider.size,
                );
                let pushed_pos = other_pos - push;
                if match collision {
                    Collision::Left | Collision::Right => true,
                    _ => false,
                } && !collider_query
                    .iter()
                    .filter(|(e, _, _, _)| e.id() != entity.id())
                    .any(|(_, other_t, _, other_c)| {
                        collide(other_t.translation, other_c.size, pushed_pos, collider.size)
                            .is_some()
                    })
                {
                    position_update.insert(other_entity, pushed_pos);
                } else {
                    let push = push_force(
                        &collision,
                        pos,
                        collider.size,
                        other_pos,
                        other_collider.size,
                    );
                    position_update.insert(entity, pos + push);
                    if let Some(velocity) = velocity_opt {
                        velocity_update.push((entity, zero_velocity(&collision, velocity)));
                    }
                }
            }
        }
    }

    for (entity, new_pos) in &position_update {
        collider_query
            .get_component_mut::<Transform>(*entity)
            .unwrap()
            .translation = *new_pos;
    }

    for (entity, new_vel) in &velocity_update {
        collider_query
            .get_component_mut::<Velocity>(*entity)
            .unwrap()
            .linvel = *new_vel;
    }

    for entity in &grounded_update {
        collider_query
            .get_component_mut::<Collider>(*entity)
            .unwrap()
            .on_ground = true;
    }

    position_update.clear();
    velocity_update.clear();
    grounded_update.clear();

    for (entity, transform, velocity_opt, collider) in
        collider_query.iter().filter(|(_, _, v, _)| v.is_some())
    {
        for (other_entity, other_transform, _, other_collider) in
            collider_query.iter().filter(|(e, _, _, c)| {
                e.id() != entity.id() && !matches!(c.r#type, ColliderType::Movable)
            })
        {
            let other_pos = other_transform.translation;
            let pos = transform.translation;
            if let Some(collision) = collide(other_pos, other_collider.size, pos, collider.size) {
                if matches!(collision, Collision::Bottom) {
                    grounded_update.push(entity);
                }
                match other_collider.r#type {
                    ColliderType::Solid => {
                        let push = push_force(
                            &collision,
                            pos,
                            collider.size,
                            other_pos,
                            other_collider.size,
                        );
                        if let Some(position) = position_update.get_mut(&entity) {
                            *position += push;
                        } else {
                            position_update.insert(entity, pos + push);
                        }
                        if let Some(velocity) = velocity_opt {
                            velocity_update.push((entity, zero_velocity(&collision, velocity)));
                        }
                        events.send(PlayerEvent::Solid(entity.id()));
                    }
                    ColliderType::Sensor => {
                        events.send(PlayerEvent::Sensor(entity.id()));
                    }
                    ColliderType::Death => {
                        events.send(PlayerEvent::Death);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    for (entity, new_pos) in position_update {
        collider_query
            .get_component_mut::<Transform>(entity)
            .unwrap()
            .translation = new_pos;
    }

    for (entity, new_vel) in velocity_update {
        collider_query
            .get_component_mut::<Velocity>(entity)
            .unwrap()
            .linvel = new_vel;
    }

    for entity in grounded_update {
        collider_query
            .get_component_mut::<Collider>(entity)
            .unwrap()
            .on_ground = true;
    }
}

fn push_force(collision: &Collision, a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Vec3 {
    match collision {
        Collision::Left => Vec2::new((b_pos.x + b_size.x / 2.0) - (a_pos.x - a_size.x / 2.0), 0.0),
        Collision::Right => Vec2::new((b_pos.x - b_size.x / 2.0) - (a_pos.x + a_size.x / 2.0), 0.0),
        Collision::Top => Vec2::new(0.0, (b_pos.y - b_size.y / 2.0) - (a_pos.y + a_size.y / 2.0)),
        Collision::Bottom => {
            Vec2::new(0.0, (b_pos.y + b_size.y / 2.0) - (a_pos.y - a_size.y / 2.0))
        }
        Collision::Inside => Vec2::ZERO,
    }
    .extend(0.0)
}

fn zero_velocity(collision: &Collision, velocity: &Velocity) -> Vec3 {
    let linvel = velocity.linvel;
    match collision {
        Collision::Left | Collision::Right => Vec3::new(0.0, linvel.y, linvel.z),
        Collision::Top | Collision::Bottom => Vec3::new(linvel.x, 0.0, linvel.z),
        Collision::Inside => Vec3::ZERO,
    }
}
