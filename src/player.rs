use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    animation::Animation,
    collide::{Collider, ColliderKind, CollisionFlags},
    map::{CellTower, BLOCK_SIZE},
    state::GameState,
    tiled_loader::WorldObject,
    velocity::{Gravity, Velocity},
};

#[derive(Deref)]
pub struct PlayerTexture(pub Handle<TextureAtlas>);

pub struct PlayerPlugin;

pub struct Latency(pub VecDeque<i32>);

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Play).with_system(load_player_resources))
            .add_system_set(
                SystemSet::on_update(GameState::Play)
                    .with_system(player_inputs)
                    .with_system(update_player)
                    .with_system(update_latency.after(player_inputs)), //.with_system(_print_player_inputs.after("map_update"))
            );
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    #[bundle]
    sprite_sheet: SpriteSheetBundle,
    player: Player,
    collider: Collider,
    velocity: Velocity,
    gravity: Gravity,
    animation: Animation,
    world_object: WorldObject,
}

impl PlayerBundle {
    pub fn new(transform: Transform, texture: Handle<TextureAtlas>) -> Self {
        Self {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: texture.clone(),
                sprite: TextureAtlasSprite {
                    custom_size: Some(Vec2::new((22.0 / 32.0) * BLOCK_SIZE, BLOCK_SIZE)),
                    ..default()
                },
                transform: Transform {
                    translation: transform.translation + Vec3::Z * 100.0,
                    ..transform
                },
                ..default()
            },
            player: Player::default(),
            collider: Collider {
                kind: ColliderKind::Movable(5.0),
                size: Vec2::new(22.0 / 32.0 * BLOCK_SIZE, BLOCK_SIZE),
                flags: CollisionFlags::empty(),
            },
            velocity: Velocity {
                drag: Vec3::new(20.0, 5.0, 0.0),
                ..default()
            },
            gravity: Gravity::default(),
            animation: Animation {
                timer: Timer::from_seconds(0.2, true),
                running: false,
            },
            world_object: WorldObject,
        }
    }
}

fn load_player_resources(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
) {
    let texture_handle = asset_server.load("robot.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(22.0, 32.0), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(PlayerTexture(texture_atlas_handle.clone()));

    commands.insert_resource(Latency(VecDeque::from(vec![0i32, 10])))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GameInput {
    Left,
    Right,
    Jump,
}

// TODO move to another file

#[derive(Component, Debug, Default)]
pub struct Player {
    // # of ticks before register TODO make it millisecionds,
    pub latency: usize,
    queue: VecDeque<Vec<GameInput>>,
}

fn player_inputs(keyboard_input: Res<Input<KeyCode>>, mut player_query: Query<&mut Player>) {
    for mut player in player_query.iter_mut() {
        let latency = player.latency;

        let mut inputs: Vec<GameInput> = keyboard_input
            .get_pressed()
            .filter_map(|input| match input {
                KeyCode::W | KeyCode::Up => {
                    if keyboard_input.just_pressed(KeyCode::W)
                        || keyboard_input.just_pressed(KeyCode::Up)
                    {
                        return Some(GameInput::Jump);
                    }
                    None
                }
                KeyCode::A | KeyCode::Left => Some(GameInput::Left),
                KeyCode::D | KeyCode::Right => Some(GameInput::Right),
                _ => None,
            })
            .collect();
        if !inputs.is_empty() {
            if player.queue.len() < latency + 1 {
                player.queue.resize(latency + 1, vec![]);
            }

            inputs.append(&mut player.queue[latency]);
            inputs.dedup();
            player.queue[latency] = inputs;
        }
    }
}

fn _print_player_inputs(player_query: Query<&Player>) {
    let player = player_query.get_single();
    println!("{:?}", player);
}

fn update_player(
    mut player_query: Query<(
        &mut Player,
        &Collider,
        &mut Velocity,
        &mut TextureAtlasSprite,
        &mut Animation,
    )>,
) {
    for (mut player, collider, mut velocity, mut player_sprite, mut animation) in
        player_query.iter_mut()
    {
        let inputs: Vec<GameInput> = player.queue.pop_front().unwrap_or_default();
        animation.running = false;
        for input in inputs {
            match input {
                GameInput::Jump => {
                    if collider.flags.contains(CollisionFlags::BOTTOM) {
                        velocity.linvel += Vec3::Y * 2300.0;
                    }
                }
                GameInput::Left => {
                    // player starts off facing right so facing left is true
                    // facing right is false
                    player_sprite.flip_x = true;
                    animation.running = true;
                    velocity.linvel += Vec3::X * -200.0
                }
                GameInput::Right => {
                    player_sprite.flip_x = false;
                    animation.running = true;
                    velocity.linvel += Vec3::X * 200.0
                }
            }
        }
    }
}

fn update_latency(
    mut player_query: Query<(&Transform, &mut Player)>,
    cell_tower_query: Query<&Transform, With<CellTower>>,
    mut latency_counter: ResMut<Latency>,
    time: Res<Time>,
) {
    latency_counter
        .0
        .push_back((time.delta_seconds() * 1000.0).floor() as i32);
    while latency_counter.0.len() > 50 {
        latency_counter.0.pop_front();
    }
    for (transform, mut player) in player_query.iter_mut() {
        let mut shortest = f32::MAX;
        for cell_tower_transform in cell_tower_query.iter() {
            shortest = cell_tower_transform
                .translation
                .distance(transform.translation)
                .min(shortest);
        }
        if shortest == f32::MAX {
            player.latency = 0;
            return;
        }
        player.latency = (shortest / (BLOCK_SIZE / 2.0)) as usize;
    }
}
