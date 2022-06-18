use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::collide::{Collider, GameEvent, TOP};
use crate::map::spawn_map;
use crate::slider::Slider;
use crate::state::GameState;
use crate::tiled_loader::WorldObject;
use crate::trigger::{Button, DoorRes};
use crate::Level;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Play)
                .with_system(handle_events)
                .after("collision")
                .with_system(button_events)
                .after("collision"),
        );
    }
}

fn button_events(
    mut buttons: Query<(&Collider, &mut Button)>,
    mut doors: Query<&mut Slider>,
    mut door_res: ResMut<DoorRes>,
) {
    for (collider, mut button) in buttons.iter_mut() {
        let mut entry = door_res.0.get_mut(&button.door).unwrap();
        let mut door = match doors.get_component_mut::<Slider>(entry.1) {
            Ok(door) => door,
            _ => return,
        };
        if collider.flags != 0 && !button.is_pressed() {
            entry.0 -= 1;
            button.pressed = true;
            if entry.0 == 0 {
                door.activated = true;
            }
        } else if collider.flags == 0 && button.is_pressed() {
            entry.0 += 1;
            button.pressed = false;
            door.activated = false;
        }
    }
}

fn handle_events(
    mut commands: Commands,
    mut events: EventReader<GameEvent>,
    mut map_query: MapQuery,
    entities: Query<Entity, With<WorldObject>>,
    asset_server: Res<AssetServer>,
    mut level: ResMut<Level>,
    input: Res<Input<KeyCode>>,
) {
    let mut is_dead = false;
    let mut won = false;
    for event in events.iter() {
        match event {
            GameEvent::Death => is_dead = true,
            GameEvent::Sensor(e) => {}
            GameEvent::Win => won = true,
        }
    }

    if is_dead {
        map_query.despawn(&mut commands, 0);
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
        spawn_map(level, commands, asset_server);
        return;
    }

    if won || input.just_pressed(KeyCode::L) {
        map_query.despawn(&mut commands, 0);
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
        level.0 += 1;
        spawn_map(level, commands, asset_server);
    }
}
