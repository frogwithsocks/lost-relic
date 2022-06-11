use crate::velocity::Velocity;
use bevy::prelude::*;
use std::collections::VecDeque;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(player_inputs)
            .add_system(update_player)
            .add_system(update_latency);
    }
}
#[derive(Component)]
struct CellTower;
fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    // TODO move to another function probably in map.rs
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(133.0 / 255.0, 193.0 / 255.0, 220.0 / 255.0),
                custom_size: Some(Vec2::new(5.0, 5.0)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::ZERO,
                ..default()
            },
            ..default()
        })
        .insert(CellTower);
    

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("robot.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::ZERO,
                ..default()
            },
            ..default()
        })
        .insert(Player {
            latency: 0,
            queue: VecDeque::new(),
        })
        .insert(Velocity {
            drag: 0.95,
            ..default()
        });
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GameInput {
    Left,
    Right,
    Jump,
}

#[derive(Component, Debug)]
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
            KeyCode::W | KeyCode::Up => Some(GameInput::Jump),
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
    println!("{:?}", player);
}

//TODO only pop when delta time is over some amount
fn update_player(mut player_query: Query<(&mut Player, &mut Velocity)>, time: Res<Time>) {
    let (mut player, mut velocity) = player_query.single_mut();
    let inputs: Vec<GameInput> = player.queue.pop_front().unwrap_or_default();
    for input in inputs {
        match input {
            GameInput::Jump => velocity.linvel += Vec3::Y * 1000.0,
            GameInput::Left => velocity.linvel += Vec3::X * -200.0,
            GameInput::Right => velocity.linvel += Vec3::X * 200.0,
        }
    }
    velocity.linvel.y -= 100.0;
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
    player.latency = (shortest / 50.0) as usize;
}
