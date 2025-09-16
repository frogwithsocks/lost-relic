use std::collections::{HashMap, HashSet};

use bevy::{
    asset::Assets,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use bitflags::bitflags;

use crate::{
    map::BLOCK_SIZE, player::Player, state::GameState, tiled_loader::TiledMap, velocity::Velocity,
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
    None,
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
    pub flags: CollisionFlags,
}

impl Collider {
    fn weight(&self) -> f32 {
        match self.kind {
            ColliderKind::Movable(w) => w,
            ColliderKind::Death => f32::INFINITY,
            ColliderKind::Sensor => f32::MAX,
            ColliderKind::Win => f32::MAX,
            ColliderKind::None => 0.0,
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
    player_entity: Query<Entity, With<Player>>,
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
        _ => return,
    };

    let player_entity = match player_entity.get_single() {
        Ok(e) => e,
        _ => return,
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
        c.flags = CollisionFlags::empty();
    }

    let mut positions: HashMap<Entity, (Vec3, CollisionFlags)> = HashMap::new();
    for (entity, transform, collider) in colliders.iter() {
        let mut position = transform.translation;
        if let Ok(velocity) = velocity_query.get_component::<Velocity>(entity) {
            position.y = (position.y + velocity.linvel.y * delta).floor();
        }
        if collider.weight() == f32::INFINITY {
            positions.insert(entity, (position, CollisionFlags::all()));
        } else {
            positions.insert(entity, (position, CollisionFlags::empty()));
        }
    }
    let mut update_velocity = HashSet::new();
    let mut update: HashSet<Entity> = movables.clone().into_iter().map(|(e, _)| e).collect();

    while !update.is_empty() {
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
                                let f = CollisionFlags::from(collision);
                                let of = f.opposite();
                                if collider.weight() < other_weight {
                                    if flags & of == CollisionFlags::empty() {
                                        flags |= f;
                                        position += push;
                                        again.insert(entity);
                                        update_velocity.insert(entity);
                                        if !other_flags.is_locked(of) {
                                            positions.insert(
                                                other_entity,
                                                (other_position, other_flags & (!of)),
                                            );
                                        }
                                        if other_flags.is_locked(f) {
                                            flags |= f.to_lock();
                                        }
                                    } else if flags.is_locked(of) {
                                        other_position -= push;
                                        again.insert(other_entity);
                                        update_velocity.insert(other_entity);
                                        positions.insert(
                                            other_entity,
                                            (other_position, other_flags | of),
                                        );
                                    } else {
                                        flags |= f;
                                        position += push;
                                        again.insert(entity);
                                        update_velocity.insert(entity);
                                        if !other_flags.is_locked(of) {
                                            positions.insert(
                                                other_entity,
                                                (other_position, other_flags & (!of)),
                                            );
                                        }
                                        if other_flags.is_locked(f) {
                                            flags |= f.to_lock();
                                        }
                                    }
                                } else if other_flags & f == CollisionFlags::empty() {
                                    other_position -= push;
                                    again.insert(other_entity);
                                    update_velocity.insert(other_entity);
                                    positions
                                        .insert(other_entity, (other_position, other_flags | of));
                                } else if other_flags.is_locked(f) {
                                    flags |= f;
                                    position += push;
                                    again.insert(entity);
                                    update_velocity.insert(entity);
                                    if !other_flags.is_locked(of) {
                                        positions.insert(
                                            other_entity,
                                            (other_position, other_flags & (!of)),
                                        );
                                    }
                                    if other_flags.is_locked(f) {
                                        flags |= f.to_lock();
                                    }
                                } else {
                                    other_position -= push;
                                    again.insert(other_entity);
                                    update_velocity.insert(other_entity);
                                    positions
                                        .insert(other_entity, (other_position, other_flags | of));
                                }
                            }
                            ColliderKind::Death => {
                                if entity == player_entity {
                                    events.send(GameEvent::Death);
                                }
                            }
                            ColliderKind::Sensor => {
                                positions.insert(
                                    other_entity,
                                    (other_position, other_flags | CollisionFlags::TOP),
                                );
                            }
                            ColliderKind::Win => {
                                events.send(GameEvent::Win);
                            }
                            _ => {}
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
            .flags |= *flags;
    }

    for (entity, transform, _) in colliders.iter() {
        let mut position = transform.translation;
        if let Ok(velocity) = velocity_query.get_component::<Velocity>(entity) {
            if velocity.linvel.x >= 0.0 {
                position.x = (position.x + velocity.linvel.x * delta).floor();
            } else {
                position.x = (position.x + velocity.linvel.x * delta).ceil();
            }
        }
        let flags = positions.get(&entity).unwrap().1;
        positions.insert(entity, (position, flags));
    }
    let mut update: HashSet<Entity> = movables.clone().into_iter().map(|(e, _)| e).collect();

    while !update.is_empty() {
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
                                let f = CollisionFlags::from(collision);
                                let of = f.opposite();
                                if collider.weight() < other_weight {
                                    if flags & of == CollisionFlags::empty() {
                                        flags |= f;
                                        position += push;
                                        again.insert(entity);
                                        update_velocity.insert(entity);
                                        if !other_flags.is_locked(of) {
                                            positions.insert(
                                                other_entity,
                                                (other_position, other_flags & (!of)),
                                            );
                                        }
                                        if other_flags.is_locked(f) {
                                            flags |= f.to_lock();
                                        }
                                    } else if flags.is_locked(of) {
                                        other_position -= push;
                                        again.insert(other_entity);
                                        update_velocity.insert(other_entity);
                                        positions.insert(
                                            other_entity,
                                            (other_position, other_flags | of),
                                        );
                                    } else {
                                        flags |= f;
                                        position += push;
                                        again.insert(entity);
                                        update_velocity.insert(entity);
                                        if !other_flags.is_locked(of) {
                                            positions.insert(
                                                other_entity,
                                                (other_position, other_flags & (!of)),
                                            );
                                        }
                                        if other_flags.is_locked(f) {
                                            flags |= f.to_lock();
                                        }
                                    }
                                } else if other_flags & f == CollisionFlags::empty() {
                                    other_position -= push;
                                    again.insert(other_entity);
                                    update_velocity.insert(other_entity);
                                    positions
                                        .insert(other_entity, (other_position, other_flags | of));
                                } else if other_flags.is_locked(f) {
                                    flags |= f;
                                    position += push;
                                    again.insert(entity);
                                    update_velocity.insert(entity);
                                    if !other_flags.is_locked(of) {
                                        positions.insert(
                                            other_entity,
                                            (other_position, other_flags & (!of)),
                                        );
                                    }
                                    if other_flags.is_locked(f) {
                                        flags |= f.to_lock();
                                    }
                                } else {
                                    other_position -= push;
                                    again.insert(other_entity);
                                    update_velocity.insert(other_entity);
                                    positions
                                        .insert(other_entity, (other_position, other_flags | of));
                                }
                            }
                            ColliderKind::Death => {
                                if entity == player_entity {
                                    events.send(GameEvent::Death);
                                }
                            }
                            ColliderKind::Sensor => {
                                positions.insert(
                                    other_entity,
                                    (other_position, other_flags | CollisionFlags::TOP),
                                );
                            }
                            ColliderKind::Win => {
                                events.send(GameEvent::Win);
                            }
                            _ => {}
                        }
                    }
                }
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
            transform.translation = (transform.translation + velocity.linvel * delta).floor();
            velocity.linvel.x -= velocity.linvel.x * drag.x * delta;
            velocity.linvel.y -= velocity.linvel.y * drag.y * delta;
        }
    }
}

bitflags! {
    pub struct CollisionFlags: u8 {
        const TOP        = 1 << 3;
        const BOTTOM     = 1 << 0;
        const RIGHT      = 1 << 2;
        const LEFT       = 1 << 1;
        const TOPLOCK    = 1 << 7;
        const BOTTOMLOCK = 1 << 4;
        const RIGHTLOCK  = 1 << 6;
        const LEFTLOCK   = 1 << 5;
    }
}

impl Default for CollisionFlags {
    fn default() -> Self {
        Self::empty()
    }
}

impl CollisionFlags {
    fn is_locked(&self, check: Self) -> bool {
        let bits = check.bits();
        assert_eq!((bits & 0x0F).count_ones(), 1);
        self.contains(CollisionFlags::from_bits(bits << 4).unwrap())
    }

    fn to_lock(self) -> Self {
        let bits = self.bits();
        assert_eq!((bits & 0x0F).count_ones(), 1);
        CollisionFlags::from_bits(bits << 4).unwrap()
    }

    fn opposite(&self) -> Self {
        let bits = self.bits();
        Self::from_bits((bits << 4).reverse_bits() | (bits & 0xF0)).unwrap()
    }
}

impl From<Collision> for CollisionFlags {
    fn from(collision: Collision) -> Self {
        match collision {
            Collision::Top | Collision::Inside => Self::TOP,
            Collision::Bottom => Self::BOTTOM,
            Collision::Right => Self::RIGHT,
            Collision::Left => Self::LEFT,
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

    fn fill(&mut self, entities: &[(Entity, &Transform, &Collider)]) {
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
