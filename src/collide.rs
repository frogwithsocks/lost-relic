use bevy::{
    asset::Assets,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use std::collections::{HashMap, HashSet};

use crate::{
    map::BLOCK_SIZE,
    tiled_loader::TiledMap,
    velocity::{Gravity, Velocity},
};

pub struct CollidePlugin;

impl Plugin for CollidePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_collisions);
    }
}

#[derive(PartialEq, Eq)]
pub enum PlayerEvent {
    Solid(u32),
    Sensor(u32),
    Death,
}

#[derive(Component)]
pub struct Solid;

#[derive(Component)]
pub struct Movable;

// TODO maybe sensors should contain a string which tells it which thing to switch on in the env
#[derive(Debug, Clone, Default, Copy)]
pub enum ColliderKind {
    #[default]
    Solid,
    Sensor,
    Movable,
    Death,
}

#[derive(Component, Clone, Default, Copy)]
pub struct Collider {
    pub size: Vec2,
    pub kind: ColliderKind,
    pub on_ground: bool,
}

fn handle_collisions(
    mut events: EventWriter<PlayerEvent>,
    mut colliders: Query<(Entity, &mut Transform, &mut Collider)>,
    mut velocity_query: Query<&mut Velocity>,
    time: Res<Time>,
    assets: Res<Assets<TiledMap>>,
    map_query: Query<&Handle<TiledMap>>,
) {
    let dimensions = assets
        .get(map_query.single())
        .map(|tm| {
            (
                tm.map.width as f32 * BLOCK_SIZE,
                tm.map.height as f32 * BLOCK_SIZE,
            )
        })
        .unwrap_or((16.0 * BLOCK_SIZE, 16.0 * BLOCK_SIZE));
    let mut partition = SpatialPartition::new(dimensions.0 as usize, dimensions.1 as usize);
    let solid: Vec<(Entity, &Transform, &Collider)> = colliders
        .iter()
        .filter(|(_, _, c)| !matches!(c.kind, ColliderKind::Movable))
        .collect();
    partition.fill(&solid);
    let delta = time.delta_seconds();
    let mut positions: HashMap<Entity, Vec3> = HashMap::new();
    let mut grounded: HashSet<Entity> = HashSet::new();
    for (entity, transform, _) in colliders.iter() {
        let mut position = transform.translation;
        if let Ok(velocity) = velocity_query.get_component::<Velocity>(entity) {
            position.y = (position.y + velocity.linvel.y * delta).floor();
        }
        positions.insert(entity, position);
    }
    let mut update_velocity = HashSet::new();
    let movables: Vec<(Entity, Collider)> = colliders
        .iter()
        .filter_map(|(e, _, c)| {
            if velocity_query.get_component::<Velocity>(e).is_ok() {
                Some((e, *c))
            } else {
                None
            }
        })
        .collect();
    let mut update: HashSet<Entity> = movables.clone().into_iter().map(|(e, _)| e).collect();

    while update.len() > 0 {
        let mut again: HashSet<Entity> = HashSet::new();
        for entity in update {
            let collider = colliders.get_component::<Collider>(entity).unwrap();
            let mut position = *positions.get(&entity).unwrap();
            let size = collider.size;
            for (other_entity, other_collider) in partition
                .possibilities(position, collider.size)
                .iter()
                .map(|(e, _, c)| (*e, *c))
                .chain(movables.iter().filter_map(|(e, c)| {
                    if e.id() != entity.id() {
                        Some((*e, *c))
                    } else {
                        None
                    }
                }))
            {
                let other_size = other_collider.size;
                let other_position = *positions.get(&other_entity).unwrap();
                if let Some(collision) = collide(other_position, other_size, position, size) {
                    if matches!(collision, Collision::Bottom) {
                        grounded.insert(entity);
                    } else {
                        grounded.insert(other_entity);
                    }
                    match collision {
                        Collision::Top | Collision::Bottom => match other_collider.kind {
                            ColliderKind::Solid => {
                                position += push_force(
                                    &collision,
                                    position,
                                    size,
                                    other_position,
                                    other_size,
                                );
                                again.insert(entity);
                                update_velocity.insert(entity);
                            }
                            ColliderKind::Movable => {
                                let push = push_force(
                                    &collision,
                                    position,
                                    size,
                                    other_position,
                                    other_size,
                                );
                                positions.insert(other_entity, other_position - push);
                                again.insert(other_entity);
                                update_velocity.insert(other_entity);
                            }
                            ColliderKind::Death => {
                                println!("death");
                                events.send(PlayerEvent::Death);
                                panic!("death");
                            }
                            _ => {}
                        },
                        _ => continue,
                    }
                }
            }
            positions.insert(entity, position);
        }
        update = again;
    }

    for entity in &update_velocity {
        if delta > 0.0 {
            let target = positions.get(&entity).unwrap().y;
            let current = colliders
                .get_component::<Transform>(*entity)
                .unwrap()
                .translation
                .y;
            let mut velocity = velocity_query
                .get_component_mut::<Velocity>(*entity)
                .unwrap();
            velocity.linvel.y = (target - current) / delta;
        }
    }

    positions.clear();
    for (entity, transform, _) in colliders.iter() {
        let mut position = transform.translation;
        if let Ok(velocity) = velocity_query.get_component::<Velocity>(entity) {
            position = (position + velocity.linvel * delta).floor();
        }
        positions.insert(entity, position);
    }
    update = movables.clone().into_iter().map(|(e, _)| e).collect();
    update_velocity.clear();

    while update.len() > 0 {
        let mut again: HashSet<Entity> = HashSet::new();
        for entity in update {
            let collider = colliders.get_component::<Collider>(entity).unwrap();
            let mut position = *positions.get(&entity).unwrap();
            let size = collider.size;
            for (other_entity, other_collider) in partition
                .possibilities(position, collider.size)
                .iter()
                .map(|(e, _, c)| (*e, *c))
                .chain(movables.iter().filter_map(|(e, c)| {
                    if e.id() != entity.id() {
                        Some((*e, *c))
                    } else {
                        None
                    }
                }))
            {
                let other_size = other_collider.size;
                let other_position = *positions.get(&other_entity).unwrap();
                if let Some(collision) = collide(other_position, other_size, position, size) {
                    match collision {
                        Collision::Right | Collision::Left => match other_collider.kind {
                            ColliderKind::Solid => {
                                position += push_force(
                                    &collision,
                                    position,
                                    size,
                                    other_position,
                                    other_size,
                                );
                                again.insert(entity);
                                update_velocity.insert(entity);
                            }
                            ColliderKind::Movable => {
                                let push = push_force(
                                    &collision,
                                    position,
                                    size,
                                    other_position,
                                    other_size,
                                );
                                positions.insert(other_entity, other_position - push);
                                again.insert(other_entity);
                                update_velocity.insert(other_entity);
                            }
                            ColliderKind::Death => {
                                println!("death");
                                events.send(PlayerEvent::Death);
                                panic!("death");
                            }
                            _ => {}
                        },
                        _ => continue,
                    }
                }
            }
            positions.insert(entity, position);
        }
        update = again;
    }

    for entity in update_velocity {
        if delta > 0.0 {
            let target = positions.get(&entity).unwrap().x;
            let current = colliders
                .get_component::<Transform>(entity)
                .unwrap()
                .translation
                .x;
            let mut velocity = velocity_query
                .get_component_mut::<Velocity>(entity)
                .unwrap();
            velocity.linvel.x = (target - current) / delta;
        }
    }

    for (entity, mut transform, mut collider) in colliders.iter_mut() {
        if let Ok(mut velocity) = velocity_query.get_component_mut::<Velocity>(entity) {
            let drag = velocity.drag;
            transform.translation = (transform.translation + velocity.linvel * delta).floor();
            velocity.linvel.x -= velocity.linvel.x * drag.x * delta;
            velocity.linvel.y -= velocity.linvel.y * drag.y * delta;

            collider.on_ground = grounded.contains(&entity);
        }
    }
}

struct SpatialPartition {
    adjust: Vec3,
    partition: Vec<Vec<Vec<(Entity, Vec3, Collider)>>>,
}

impl SpatialPartition {
    const CELL_SIZE: f32 = BLOCK_SIZE * 4.0;
    fn new(real_width: usize, real_height: usize) -> Self {
        let width =
            (real_width as f32 / Self::CELL_SIZE).ceil() as usize * 2;
        let height =
            (real_height as f32 / Self::CELL_SIZE).ceil() as usize * 2;
        let adjust = Vec3::new(real_width as f32 / 2.0, real_height as f32 / 2.0, 0.0);
        let mut partition = Vec::with_capacity(width);
        for i in 0..width {
            partition.push(Vec::with_capacity(height));
            for _ in 0..height {
                partition[i].push(Vec::new());
            }
        }

        Self { adjust, partition }
    }

    // ---------
    // | c | a |
    // ---------
    // | b | d |
    // ---------

    fn fill(&mut self, entities: &Vec<(Entity, &Transform, &Collider)>) {
        for (entity, transform, collider) in entities {
            let (a, b, c, d) = self.spatial_index(transform.translation, collider.size);
            self.partition[a.0][a.1].push((*entity, transform.translation, **collider));
            if b != c && b != d {
                self.partition[b.0][b.1].push((*entity, transform.translation, **collider));
            }
            if c != b && c != a {
                self.partition[c.0][c.1].push((*entity, transform.translation, **collider));
            }
            if d != b && d != a {
                self.partition[d.0][d.1].push((*entity, transform.translation, **collider));
            }
        }
    }

    // TODO: ignore indices outside of bounds?
    fn possibilities(&self, position: Vec3, size: Vec2) -> Vec<&(Entity, Vec3, Collider)> {
        let (a, b, c, d) = self.spatial_index(position, size);
        self.partition[a.0][a.1]
            .iter()
            .chain(self.partition[b.0][b.1].iter())
            .chain(self.partition[c.0][c.1].iter())
            .chain(self.partition[d.0][d.1].iter())
            .collect()
    }

    fn spatial_index(
        &self,
        position: Vec3,
        size: Vec2,
    ) -> (
        (usize, usize),
        (usize, usize),
        (usize, usize),
        (usize, usize),
    ) {
        let position = position + self.adjust;
        let size = size.extend(0.0);
        let hx = Vec3::new(size.x / 2.0, 0.0, 0.0);
        let hy = Vec3::new(0.0, size.y / 2.0, 0.0);
        let (a, b, c, d) = (
            ((position - size / 2.0) / Self::CELL_SIZE).floor(),
            ((position + size / 2.0) / Self::CELL_SIZE).floor(),
            ((position + hy - hx) / Self::CELL_SIZE).floor(),
            ((position - hy + hx) / Self::CELL_SIZE).floor(),
        );

        (
            (a.x as usize, a.y as usize),
            (b.x as usize, b.y as usize),
            (c.x as usize, c.y as usize),
            (d.x as usize, d.y as usize),
        )
    }
}

fn push_force(collision: &Collision, a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Vec3 {
    (match collision {
        Collision::Left => Vec2::new((b_pos.x + b_size.x / 2.0) - (a_pos.x - a_size.x / 2.0), 0.0),
        Collision::Right => Vec2::new((b_pos.x - b_size.x / 2.0) - (a_pos.x + a_size.x / 2.0), 0.0),
        Collision::Top => Vec2::new(0.0, (b_pos.y - b_size.y / 2.0) - (a_pos.y + a_size.y / 2.0)),
        Collision::Bottom => {
            Vec2::new(0.0, (b_pos.y + b_size.y / 2.0) - (a_pos.y - a_size.y / 2.0))
        }
        Collision::Inside => Vec2::ZERO,
    }
    .extend(0.0))
    .round()
}

fn zero_velocity(collision: &Collision, linvel: Vec3) -> Vec3 {
    match collision {
        Collision::Left | Collision::Right => Vec3::new(0.0, linvel.y, linvel.z),
        Collision::Top | Collision::Bottom => Vec3::new(linvel.x, 0.0, linvel.z),
        Collision::Inside => panic!("impossible state"),
    }
}
