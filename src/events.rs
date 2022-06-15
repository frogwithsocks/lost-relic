use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::triggers::{ButtonRes, DoorRes, Button};
use crate::collide::{GameEvent};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_events);
    }
}

fn handle_events(mut commands: Commands, mut events: EventReader<GameEvent>, mut buttons: ResMut<ButtonRes>, mut doors: ResMut<DoorRes>, mut map_query: MapQuery) {
    let mut is_dead = false;
    for event in events.iter() {
        match event {
            GameEvent::Death => is_dead = true,
            GameEvent::Sensor(id) => {

            },
        }
    }

    if is_dead {
        map_query.despawn(&mut commands, 0);
    }
}