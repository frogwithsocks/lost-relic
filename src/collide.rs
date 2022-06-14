use bevy::{
    asset::Assets,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use bevy_ecs_tilemap::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::{map::BLOCK_SIZE, tiled_loader::TiledMap, velocity::Velocity};

pub struct CollidePlugin;

impl Plugin for CollidePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(check_collisions);
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
    assets: Res<Assets<TiledMap>>,
    map_query: Query<&Handle<TiledMap>>,
) {
    let dimensions = assets.get(map_query.single()).map(|tm| Vec2::new(tm.map.width as f32, tm.map.height as f32)).unwrap_or(Vec2::splat(16.0)) * BLOCK_SIZE;
    let mut partition = SpatialPartition::new(dimensions.x as usize, dimensions.y as usize);
    partition.fill(&collider_query);
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
            for other_entity in partition
                .possibilities(pos.truncate(), size)
                .into_iter()
                .filter(|e| e.id() != entity.id())
            {
                let other_transform = collider_query
                    .get_component::<Transform>(*other_entity)
                    .unwrap();
                let other_collider = collider_query
                    .get_component::<Collider>(*other_entity)
                    .unwrap();
                if !update_position.contains_key(other_entity) {
                    update_position.insert(*other_entity, other_transform.translation);
                }
                let other_pos = update_position.get_mut(&other_entity).unwrap();
                let other_size = other_collider.size;
                if let Some(collision) = collide(*other_pos, other_size * 0.99, pos, size * 0.99) {
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
                        ColliderKind::Sensor => {
                            events.send(PlayerEvent::Sensor(other_entity.id()));
                        }
                        ColliderKind::Death => {
                            events.send(PlayerEvent::Death);
                        }
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
                            again.insert(*other_entity);
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

        for (entity, linvel) in update_velocity {
            collider_query
                .get_component_mut::<Velocity>(entity)
                .unwrap()
                .linvel = linvel;
        }

        update = again;
    }

    for (entity, _, _, mut collider) in collider_query.iter_mut() {
        collider.on_ground = update_grounded.contains(&entity);
    }
}

struct SpatialPartition {
    adjust: Vec2,
    width: usize,
    height: usize,
    partition: Vec<Vec<HashSet<Entity>>>,
}

impl SpatialPartition {
    const CELL_SIZE: f32 = BLOCK_SIZE;
    fn new(real_width: usize, real_height: usize) -> Self {
        let width = (real_width as f32 / Self::CELL_SIZE) as usize * 2;
        let height = (real_height as f32 / Self::CELL_SIZE) as usize * 2;
        let adjust = Vec2::new(real_width as f32 / 2.0, real_height as f32 / 2.0);
        let mut partition: Vec<Vec<HashSet<Entity>>> = Vec::with_capacity(width);
        for i in 0..width {
            partition.push(Vec::with_capacity(height));
            for _ in 0..height {
                partition[i].push(HashSet::new());
            }
        }

        Self {
            adjust,
            width,
            height,
            partition,
        }
    }

    fn fill(&mut self, entities: &Query<(Entity, &mut Transform, Option<&mut Velocity>, &mut Collider)>) {
        for (entity, transform, _, collider) in entities.iter() {
            let (a, b, c, d) = self.spatial_index(transform.translation.truncate(), collider.size);
            self.partition[a.0][a.1].insert(entity);
            self.partition[b.0][b.1].insert(entity);
            self.partition[c.0][c.1].insert(entity);
            self.partition[d.0][d.1].insert(entity);
        }
    }

    fn clear(&mut self) {
        for i in 0..self.width {
            for j in 0..self.height {
                self.partition[i][j].clear();
            }
        }
    }

    fn possibilities(&self, position: Vec2, size: Vec2) -> Vec<&Entity> {
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
        position: Vec2,
        size: Vec2,
    ) -> (
        (usize, usize),
        (usize, usize),
        (usize, usize),
        (usize, usize),
    ) {
        let position = position + self.adjust;
        let hx = Vec2::new(size.x / 2.0, 0.0);
        let hy = Vec2::new(0.0, size.y / 2.0);
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
        Collision::Bottom => Vec2::new(
            0.0,
            (b_pos.y + b_size.y / 2.0) - (a_pos.y - a_size.y / 2.0) + 2.0,
        ),
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
