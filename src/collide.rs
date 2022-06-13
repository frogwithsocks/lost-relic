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
        app.add_system(check_collisions.after("velocity"));
    }
}

pub enum PlayerEvent {
    Solid(u32),
    Sensor(u32),
    Death,
}

// TODO maybe sensors should contain a string which tells it which thing to switch on in the env
#[derive(Debug, Clone, Default)]
pub enum ColliderKind {
    #[default]
    Solid,
    Sensor,
    Movable,
    Death,
}

#[derive(Component, Clone, Default)]
pub struct Collider {
    pub size: Vec2,
    pub kind: ColliderKind,
    pub on_ground: bool,
}

fn check_collisions(
    mut events: EventWriter<PlayerEvent>,
    mut collider_query: Query<(Entity, &mut Transform, Option<&mut Velocity>, &mut Collider)>,
) {
    let mut update: HashSet<Entity> = collider_query
        .iter()
        .filter(|(_, _, v, _)| v.is_some())
        .map(|(e, _, _, _)| e)
        .collect();
    let mut update_grounded = HashSet::new();
    while update.len() > 0 {
        let mut again = HashSet::new();
        let mut update_position = HashMap::new();
        let mut update_velocity = HashMap::new();
        for entity in update {
            let transform = collider_query.get_component::<Transform>(entity).unwrap();
            let opt_velocity = collider_query.get_component::<Velocity>(entity);
            let collider = collider_query.get_component::<Collider>(entity).unwrap();
            let mut pos = transform.translation;
            let size = collider.size;
            for (other_entity, other_transform, _, other_collider) in collider_query
                .iter()
                .filter(|(e, _, _, _)| e.id() != entity.id())
            {
                if !update_position.contains_key(&other_entity) {
                    update_position.insert(other_entity, other_transform.translation);
                }
                let other_pos = update_position.get_mut(&other_entity).unwrap();
                let other_size = other_collider.size;
                if let Some(collision) = collide(*other_pos, other_size, pos, size) {
                    if matches!(collision, Collision::Bottom) {
                        update_grounded.insert(entity);
                    }
                    match other_collider.kind {
                        ColliderKind::Solid => {
                            let push = push_force(&collision, pos, size, *other_pos, other_size);
                            pos += push;
                            if let Ok(velocity) = opt_velocity {
                                if let Some(previous) = update_velocity.get_mut(&entity) {
                                    *previous = zero_velocity(&collision, *previous);
                                } else {
                                    update_velocity
                                        .insert(entity, zero_velocity(&collision, velocity.linvel));
                                }
                            }
                            again.insert(entity);
                        }
                        ColliderKind::Sensor => {}
                        ColliderKind::Death => {}
                        ColliderKind::Movable => {
                            let push = push_force(
                                &opposite(&collision),
                                *other_pos,
                                other_size,
                                pos,
                                size,
                            );
                            *other_pos += push;
                            if let Ok(velocity) = opt_velocity {
                                if let Some(previous) = update_velocity.get_mut(&entity) {
                                    *previous = zero_velocity(&collision, *previous);
                                } else {
                                    update_velocity
                                        .insert(entity, zero_velocity(&collision, velocity.linvel));
                                }
                            }
                            again.insert(other_entity);
                        }
                    }
                }
            }
            update_position.insert(entity, pos);
        }

        for (entity, position) in update_position {
            collider_query
                .get_component_mut::<Transform>(entity)
                .unwrap()
                .translation = position;
        }

        for (entity, velocity) in update_velocity {
            collider_query
                .get_component_mut::<Velocity>(entity)
                .unwrap()
                .linvel = velocity;
        }

        update = again;
    }

    for (entity, _, _, mut collider) in collider_query.iter_mut() {
        collider.on_ground = update_grounded.contains(&entity);
    }
}

fn push_force(collision: &Collision, a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Vec3 {
    (match collision {
        Collision::Left => Vec2::new((b_pos.x + b_size.x / 2.0) - (a_pos.x - a_size.x / 2.0), 0.0),
        Collision::Right => Vec2::new((b_pos.x - b_size.x / 2.0) - (a_pos.x + a_size.x / 2.0), 0.0),
        Collision::Top => Vec2::new(0.0, (b_pos.y - b_size.y / 2.0) - (a_pos.y + a_size.y / 2.0)),
        Collision::Bottom => {
            Vec2::new(0.0, (b_pos.y + b_size.y / 2.0) - (a_pos.y - a_size.y / 2.0) + 1.0)
        }
        Collision::Inside => Vec2::ZERO,
    }
    .extend(0.0))
    .floor()
}

fn zero_velocity(collision: &Collision, linvel: Vec3) -> Vec3 {
    match collision {
        Collision::Left | Collision::Right => Vec3::new(0.0, linvel.y, linvel.z),
        Collision::Top | Collision::Bottom => Vec3::new(linvel.x, 0.0, linvel.z),
        Collision::Inside => panic!("impossible state"),
    }
}

fn opposite(collision: &Collision) -> Collision {
    match collision {
        Collision::Left => Collision::Right,
        Collision::Right => Collision::Left,
        Collision::Bottom => Collision::Top,
        Collision::Top => Collision::Bottom,
        Collision::Inside => Collision::Inside,
    }
}
