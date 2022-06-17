use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::Level;
use crate::collide::GameEvent;
use crate::map::spawn_map;
use crate::state::GameState;
use crate::tiled_loader::WorldObject;
use crate::trigger::{Button, ButtonRes, DoorRes};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_update(GameState::Play)
                    .with_system(handle_events)
            );
    }
}

fn handle_events(
    mut commands: Commands,
    mut events: EventReader<GameEvent>,
    mut buttons: ResMut<ButtonRes>,
    mut doors: ResMut<DoorRes>,
    mut map_query: MapQuery,
    entities: Query<Entity, With<WorldObject>>,
    asset_server: Res<AssetServer>,
    mut level: ResMut<Level>,
    input :Res<Input<KeyCode>>
) {
    let mut is_dead = false;
    let mut won = false;
    for event in events.iter() {
        match event {
            GameEvent::Death => is_dead = true,
            GameEvent::Sensor(id) => {
                if let Some(button) = buttons.0.get_mut(&id) {
                    let door = doors.0.get_mut(&button.door).unwrap();
                    button.toggle();
                    *door -= button.is_pressed() as usize;
                }
            }
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

    if won || input.just_pressed(KeyCode::L)  {
        map_query.despawn(&mut commands, 0);
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
        level.0 += 1;
        spawn_map(level, commands, asset_server);
    }
}
