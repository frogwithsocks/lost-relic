use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::triggers::{ButtonRes, DoorRes, Button};
use crate::collide::{GameEvent};
use crate::tiled_loader::WorldObject;
use crate::map::spawn_map;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_events);
    }
}

fn handle_events(mut commands: Commands, mut events: EventReader<GameEvent>, mut buttons: ResMut<ButtonRes>, mut doors: ResMut<DoorRes>, mut map_query: MapQuery, mut entities: Query<Entity, With<WorldObject>>, asset_server: Res<AssetServer>) {
    let mut is_dead = false;
    for event in events.iter() {
        match event {
            GameEvent::Death => is_dead = true,
            GameEvent::Sensor(id) => {
                if let Some(button) = buttons.0.get_mut(&id) {
                    let door = doors.0.get_mut(&button.door).unwrap();
                    button.toggle();
                    *door -= button.is_pressed() as usize;
                }
            },
        }
    }

    if is_dead {
        map_query.despawn(&mut commands, 0);
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
        spawn_map(commands, asset_server);
    }
}