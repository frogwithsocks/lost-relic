use crate::{
    animation::Animation,
    collide::{Collider, ColliderKind},
    map::{CellTower, BLOCK_SIZE},
    velocity::{Gravity, Velocity},
};

use bevy::prelude::*;
use std::collections::VecDeque;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(player_inputs.label("player"))
            .add_system(update_player.label("player"))
            .add_system(update_latency.label("player").after("map_colliders"));
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("robot.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(22.0, 32.0), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::new((22.0 / 32.0) * BLOCK_SIZE, BLOCK_SIZE)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, BLOCK_SIZE, 100.0),
                ..default()
            },
            ..default()
        })
        .insert(Player::default())
        .insert(Collider {
            kind: ColliderKind::Movable,
            size: Vec2::new(22.0 / 32.0 * BLOCK_SIZE, BLOCK_SIZE),
            on_ground: false,
        })
        .insert(Velocity {
            drag: Vec3::new(20.0, 10.0, 0.0),
            ..default()
        })
        .insert(Gravity::default())
        .insert(Animation {
            timer: Timer::from_seconds(0.2, true),
            running: false,
        });
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
    latency: usize,
    queue: VecDeque<Vec<GameInput>>,
}

fn player_inputs(keyboard_input: Res<Input<KeyCode>>, mut player_query: Query<&mut Player>) {
    let mut player = player_query.single_mut();
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

    if player.queue.len() < latency + 1 {
        player.queue.resize(latency + 1, vec![]);
    }

    inputs.append(&mut player.queue[latency]);
    inputs.dedup();
    player.queue[latency] = inputs;
}

fn print_player_inputs(player_query: Query<&Player>) {
    let player = player_query.single();
    println!("{:?}", player.queue);
}

//TODO only pop when delta time is over some amount
fn update_player(
    mut player_query: Query<(
        &mut Player,
        &mut Collider,
        &mut Velocity,
        &mut TextureAtlasSprite,
        &mut Animation,
    )>,
) {
    let (mut player, mut collider, mut velocity, mut player_sprite, mut animation) =
        player_query.single_mut();
    let inputs: Vec<GameInput> = player.queue.pop_front().unwrap_or_default();
    animation.running = false;
    for input in inputs {
        match input {
            GameInput::Jump => {
                if collider.on_ground {
                    velocity.linvel += Vec3::Y * 2600.0;
                }
            }
            GameInput::Left => {
                // player starts off facing right so facing left is true
                // facing right is false
                player_sprite.flip_x = true;
                animation.running = true;
                velocity.linvel += Vec3::X * -150.0
            }
            GameInput::Right => {
                player_sprite.flip_x = false;
                animation.running = true;
                velocity.linvel += Vec3::X * 150.0
            }
        }
    }
    velocity.linvel.y -= 125.0;
}

fn update_latency(
    mut player_query: Query<(&Transform, &mut Player)>,
    cell_tower_query: Query<&Transform, With<CellTower>>,
) {
    let (transform, mut player) = player_query.single_mut();
    let mut shortest = f32::MAX;
    for cell_tower_transform in cell_tower_query.iter() {
        shortest = cell_tower_transform
            .translation
            .distance(transform.translation)
            .min(shortest);
    }
    if shortest == f32::MAX {
        player.latency = 0;
        return
    }
    player.latency = (shortest / 50.0) as usize;
}
