use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::triggers::{ButtonRes, DoorRes, Button};
use crate::collide::{GameEvent};

fn handle_events(mut events: EventReader<GameEvent>, mut buttons: ResMut<ButtonRes>, mut doors: ResMut<DoorRes>, mut map_query: MapQuery) {
    let mut is_dead = false;
    for event in events.iter() {
        match event {
            GameEvent::Death => is_dead = true,
            GameEvent::Sensor(id) => {

            },
        }
    }
}

/*
fn death_update_map(
    mut player_events: EventReader<GamerEvent>,
    mut commands: Commands,

) {
    if player_events
        .iter()
        .filter(|e| **e == PlayerEvent::Death)
        .count()
        > 0
    {
        map_query.despawn(&mut commands, 0u16);
    }
}
*/