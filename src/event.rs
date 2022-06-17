use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::Level;
use crate::collide::{Collider, GameEvent, TOP};
use crate::map::spawn_map;
use crate::state::GameState;
use crate::tiled_loader::WorldObject;
use crate::trigger::{Button, DoorRes};
use crate::slider::Slider;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_update(GameState::Play)
                    .with_system(handle_events)
                    .after("collision")
            );
    }
}

fn button_events(
    mut events: EventWriter<GameEvent>,
    buttons: Query<(Entity, &Collider, &Button)>
) {
    for (entity, collider, button) in buttons.iter() {
        if collider.flags & TOP != 0 {
            events.send(GameEvent::Sensor(entity));
        }
    }
}

fn handle_events(
    mut commands: Commands,
    mut events: EventReader<GameEvent>,
    mut buttons: Query<&mut Button>,
    mut doors: Query<&mut Slider>,
    mut door_map: ResMut<DoorRes>,
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
            GameEvent::Sensor(e) => {
                if let Ok(mut button) = buttons.get_component_mut::<Button>(*e) {
                    let (press, door_entity) = door_map.0.get_mut(&button.door).unwrap();
                    button.toggle();
                    *press -= button.is_pressed() as usize;
                    let mut door = doors.get_component_mut::<Slider>(*door_entity).unwrap();
                    if *press == 0 {
                        door.activated = true;
                    } else {
                        door.activated = false;
                    }
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
