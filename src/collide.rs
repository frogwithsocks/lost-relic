use std::collections::{HashMap, HashSet};

use bevy::{
    asset::Assets,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

use crate::{
    map::BLOCK_SIZE,
    state::GameState,
    tiled_loader::TiledMap,
    velocity::{Velocity},
};

pub struct CollidePlugin;

impl Plugin for CollidePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Play)
                .with_system(handle_collisions)
                .label("collision"),
        );
    }
}

#[derive(PartialEq, Eq)]
pub enum GameEvent {
    Death,
    Win,
}

#[derive(Component)]
pub struct Solid;

#[derive(Component)]
pub struct Movable;

// TODO maybe sensors should contain a string which tells it which thing to switch on in the env
#[derive(Debug, Clone, Copy)]
pub enum ColliderKind {
    Movable(f32),
    Death,
    Sensor,
    Win,
}

impl Default for ColliderKind {
    fn default() -> Self {
        Self::Movable(f32::INFINITY)
    }
}

#[derive(Component, Clone, Default, Copy)]
pub struct Collider {
    pub size: Vec2,
    pub kind: ColliderKind,
    pub flags: u8,
}

impl Collider {
    fn weight(&self) -> f32 {
        match self.kind {
            ColliderKind::Movable(w) => w,
            ColliderKind::Death => f32::INFINITY,
            ColliderKind::Sensor => f32::MAX,
            ColliderKind::Win => f32::MAX,
        }
    }
}

#[derive(Eq, PartialEq)]
enum Axis {
    X,
    Y,
}

impl From<&Collision> for Axis {
    fn from(collision: &Collision) -> Self {
        match collision {
            Collision::Right | Collision::Left => Axis::X,
            Collision::Top | Collision::Bottom | Collision::Inside => Axis::Y,
        }
    }
}

impl From<usize> for Axis {
    fn from(i: usize) -> Self {
        match i {
            0 => Axis::X,
            1 => Axis::Y,
            _ => panic!("invalid axis"),
        }
    }
}

fn handle_collisions(
    mut events: EventWriter<GameEvent>,
    mut colliders: Query<(Entity, &mut Transform, &mut Collider)>,
    mut velocity_query: Query<&mut Velocity>,
    time: Res<Time>,
    assets: Res<Assets<TiledMap>>,
    map_query: Query<&Handle<TiledMap>>,
) {
    let dimensions = match map_query.get_single() {
        Ok(handle) => assets
            .get(handle)
            .map(|tm| {
                (
                    tm.map.width as f32 * BLOCK_SIZE,
                    tm.map.height as f32 * BLOCK_SIZE,
                )
            })
            .unwrap_or((16.0 * BLOCK_SIZE, 16.0 * BLOCK_SIZE)),
        Err(_) => return,
    };

    let mut partition = SpatialPartition::new(dimensions.0 as usize, dimensions.1 as usize);
    let solid: Vec<(Entity, &Transform, &Collider)> = colliders
        .iter()
        .filter(|(_, _, c)| c.weight() >= 1000.0)
        .collect();
    partition.fill(&solid);
    let movables: Vec<(Entity, Collider)> = colliders
        .iter()
        .filter_map(|(e, _, c)| {
            if c.weight() < 1000.0 {
                Some((e, *c))
            } else {
                None
            }
        })
        .collect();
    let delta = time.delta_seconds();

    for (_, _, mut c) in colliders.iter_mut() {
        c.flags = 0;
    }

    let mut positions: HashMap<Entity, (Vec3, u8)> = HashMap::new();
    for (entity, transform, collider) in colliders.iter() {
        let mut position = transform.translation;
        if let Ok(velocity) = velocity_query.get_component::<Velocity>(entity) {
            position.y = (position.y + velocity.linvel.y * delta).floor();
        }
        if collider.weight() == f32::INFINITY {
            positions.insert(entity, (position, 0b1111));
        } else {
            positions.insert(entity, (position, 0b0000));
        }
    }
    let mut update_velocity = HashSet::new();
    let mut update: HashSet<Entity> = movables.clone().into_iter().map(|(e, _)| e).collect();

    while update.len() > 0 {
        let mut again: HashSet<Entity> = HashSet::new();
        for entity in update {
            let collider = colliders.get_component::<Collider>(entity).unwrap();
            let state = *positions.get(&entity).unwrap();
            let mut position = state.0;
            let mut flags = state.1;
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
                let other_state = *positions.get(&other_entity).unwrap();
                let mut other_position = other_state.0;
                let other_flags = other_state.1;
                if let Some(collision) = collide(other_position, other_size, position, size) {
                    if matches!(collision, Collision::Top)
                        | matches!(collision, Collision::Bottom)
                        | matches!(collision, Collision::Inside)
                    {
                        match other_collider.kind {
                            ColliderKind::Movable(other_weight) => {
                                let push = push_force(
                                    &collision,
                                    position,
                                    size,
                                    other_position,
                                    other_size,
                                );
                                let f = flag(&collision);
                                let of = opposite(f);
                                flags |= f;
                                if (collider.weight() <= other_weight && flags & of == 0)
                                    || other_flags & f != 0
                                {
                                    position += push;
                                    again.insert(entity);
                                    update_velocity.insert(entity);
                                } else {
                                    other_position -= push;
                                    again.insert(other_entity);
                                    update_velocity.insert(other_entity);
                                }
                                positions.insert(other_entity, (other_position, other_flags | of));
                            }
                            ColliderKind::Death => {
                                events.send(GameEvent::Death);
                            }
                            ColliderKind::Sensor => {
                                positions.insert(other_entity, (other_position, other_flags | TOP));
                            }
                            ColliderKind::Win => {
                                events.send(GameEvent::Win);
                            }
                        }
                    }
                }
            }
            positions.insert(entity, (position, flags));
        }
        update = again;
    }

    for (entity, (position, flags)) in &positions {
        let entity = *entity;
        if delta > 0.0 {
            if let Ok(mut velocity) = velocity_query.get_component_mut::<Velocity>(entity) {
                let target = position.y;
                let current = colliders
                    .get_component::<Transform>(entity)
                    .unwrap()
                    .translation
                    .y;
                velocity.linvel.y = (target - current) / delta;
            }
        }
        colliders
            .get_component_mut::<Collider>(entity)
            .unwrap()
            .flags |= flags;
    }

    for (entity, transform, _) in colliders.iter() {
        let mut position = transform.translation;
        if let Ok(velocity) = velocity_query.get_component::<Velocity>(entity) {
            position.x = (position.x + velocity.linvel.x * delta).floor();
        }
        let flags = positions.get(&entity).unwrap().1;
        positions.insert(entity, (position, flags));
    }
    let mut update: HashSet<Entity> = movables.clone().into_iter().map(|(e, _)| e).collect();

    while update.len() > 0 {
        let mut again: HashSet<Entity> = HashSet::new();
        for entity in update {
            let collider = colliders.get_component::<Collider>(entity).unwrap();
            let state = *positions.get(&entity).unwrap();
            let mut position = state.0;
            let mut flags = state.1;
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
                let other_state = *positions.get(&other_entity).unwrap();
                let mut other_position = other_state.0;
                let other_flags = other_state.1;
                //if other_size.y > 0.0 {
                if let Some(collision) = collide(other_position, other_size, position, size) {
                    if matches!(collision, Collision::Right)
                        | matches!(collision, Collision::Left)
                        | matches!(collision, Collision::Inside)
                    {
                        match other_collider.kind {
                            ColliderKind::Movable(other_weight) => {
                                let push = push_force(
                                    &collision,
                                    position,
                                    size,
                                    other_position,
                                    other_size,
                                );
                                let f = flag(&collision);
                                let of = opposite(f);
                                flags |= f;
                                if (collider.weight() <= other_weight && flags & of == 0)
                                    || other_flags & f != 0
                                {
                                    position += push;
                                    again.insert(entity);
                                    update_velocity.insert(entity);
                                } else {
                                    other_position -= push;
                                    again.insert(other_entity);
                                    update_velocity.insert(other_entity);
                                }
                                positions.insert(other_entity, (other_position, other_flags | of));
                            }
                            ColliderKind::Death => {
                                events.send(GameEvent::Death);
                            }
                            ColliderKind::Sensor => {
                                positions.insert(other_entity, (other_position, other_flags | TOP));
                            }
                            ColliderKind::Win => {
                                events.send(GameEvent::Win);
                            }
                        }
                    }
                }
                //}
            }
            positions.insert(entity, (position, flags));
        }
        update = again;
    }

    for (entity, (position, flags)) in positions {
        if delta > 0.0 {
            if let Ok(mut velocity) = velocity_query.get_component_mut::<Velocity>(entity) {
                let target = position.x;
                let current = colliders
                    .get_component::<Transform>(entity)
                    .unwrap()
                    .translation
                    .x;
                velocity.linvel.x = (target - current) / delta;
            }
        }
        colliders
            .get_component_mut::<Collider>(entity)
            .unwrap()
            .flags |= flags;
    }

    for (entity, mut transform, _) in colliders.iter_mut() {
        if let Ok(mut velocity) = velocity_query.get_component_mut::<Velocity>(entity) {
            let drag = velocity.drag;
            transform.translation = (transform.translation + velocity.linvel * delta).ceil();
            velocity.linvel.x -= velocity.linvel.x * drag.x * delta;
            velocity.linvel.y -= velocity.linvel.y * drag.y * delta;
        }
    }
}

// consider adding bitflags as a dependeny to make this cleaner?
pub const TOP: u8 = 0b1000;
pub const BOTTOM: u8 = 0b0001;
pub const RIGHT: u8 = 0b0100;
pub const LEFT: u8 = 0b0010;

fn flag(collision: &Collision) -> u8 {
    match collision {
        Collision::Top | Collision::Inside => TOP,
        Collision::Bottom => BOTTOM,
        Collision::Right => RIGHT,
        Collision::Left => LEFT,
    }
}

fn opposite(flag: u8) -> u8 {
    (flag << 4).reverse_bits()
}

struct SpatialPartition {
    adjust: Vec3,
    partition: Vec<Vec<Vec<(Entity, Vec3, Collider)>>>,
}

impl SpatialPartition {
    const CELL_SIZE: f32 = BLOCK_SIZE * 4.0;
    fn new(real_width: usize, real_height: usize) -> Self {
        let width = (real_width as f32 / Self::CELL_SIZE).ceil() as usize * 4;
        let height = (real_height as f32 / Self::CELL_SIZE).ceil() as usize * 4;
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
            let position = transform.translation;
            let (min_x, max_x, min_y, max_y) = self.spatial_index(position, collider.size);
            for i in min_x..=max_x {
                for j in min_y..=max_y {
                    self.partition[i][j].push((*entity, transform.translation, **collider));
                }
            }
        }
    }

    // TODO: ignore indices outside of bounds?
    fn possibilities(&self, position: Vec3, size: Vec2) -> Vec<&(Entity, Vec3, Collider)> {
        let mut nearby = Vec::new();
        let (min_x, max_x, min_y, max_y) = self.spatial_index(position, size);
        for i in min_x..=max_x {
            for j in min_y..=max_y {
                for item in &self.partition[i][j] {
                    nearby.push(item);
                }
            }
        }

        nearby
    }

    fn spatial_index(&self, position: Vec3, size: Vec2) -> (usize, usize, usize, usize) {
        let position = position + self.adjust;
        let size = size.extend(0.0);
        let (a, b) = (
            ((position - size / 2.0) / Self::CELL_SIZE).floor(),
            ((position + size / 2.0) / Self::CELL_SIZE).floor(),
        );

        // (min_x, max_x, min_y, max_y)
        (a.x as usize, b.x as usize, a.y as usize, b.y as usize)
    }
}

fn push_force(collision: &Collision, a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> Vec3 {
    (match collision {
        Collision::Left => Vec2::new((b_pos.x + b_size.x / 2.0) - (a_pos.x - a_size.x / 2.0), 0.0),
        Collision::Right => Vec2::new((b_pos.x - b_size.x / 2.0) - (a_pos.x + a_size.x / 2.0), 0.0),
        Collision::Top => Vec2::new(0.0, (b_pos.y - b_size.y / 2.0) - (a_pos.y + a_size.y / 2.0)),
        Collision::Bottom | Collision::Inside => {
            Vec2::new(0.0, (b_pos.y + b_size.y / 2.0) - (a_pos.y - a_size.y / 2.0))
        }
    }
    .extend(0.0))
    .round()
}
