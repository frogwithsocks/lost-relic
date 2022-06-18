use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    collide::{Collider, GameEvent},
    map::spawn_map,
    slider::Slider,
    state::GameState,
    tiled_loader::WorldObject,
    trigger::{Button, DoorRes},
    Level,
};

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

fn reset_buttons_and_doors(
    mut buttons: Query<(&mut Collider, &mut Button)>,
    mut doors: Query<(&mut Transform, &mut Collider, &mut Sprite, &mut Slider)>,
    mut door_res: ResMut<DoorRes>,
) {
    for (_, button) in buttons.iter() {
        door_res.0.get_mut(&button.door).unwrap().0 = 0;
    }
    for (mut collider, mut button) in buttons.iter_mut() {
        collider.flags = 0;
        button.pressed = false;
        door_res.0.get_mut(&button.door).unwrap().0 += 1;
    }
    for (mut transform, mut collider, mut sprite, mut slider) in doors.iter_mut() {
        slider.activated = false;
        collider.size.y += slider.extent;
        sprite.custom_size = sprite
            .custom_size
            .map(|size| Vec2::new(size.x, size.y + slider.extent));
        transform.translation.y += slider.extent / 2.0;
        slider.extent = 0.0;
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
            button.toggle();
            if entry.0 == 0 {
                door.activated = true;
            }
        } else if collider.flags == 0 && button.is_pressed() {
            entry.0 += 1;
            button.toggle();
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
    mut buttons: Query<(&mut Collider, &mut Button)>,
    mut doors: Query<(&mut Transform, &mut Collider, &mut Sprite, &mut Slider)>,
    mut door_res: ResMut<DoorRes>,
) {
    let mut is_dead = false;
    let mut won = false;
    for event in events.iter() {
        match event {
            GameEvent::Death => is_dead = true,
            GameEvent::Win => won = true,
        }
    }

    if is_dead || input.just_pressed(KeyCode::R) {
        map_query.despawn(&mut commands, 0);
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
        spawn_map(level, commands, asset_server);
        reset_buttons_and_doors(buttons, doors, door_res);
        return;
    }

    if won || input.just_pressed(KeyCode::L) {
        map_query.despawn(&mut commands, 0);
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
        level.0 += 1;
        spawn_map(level, commands, asset_server);
        reset_buttons_and_doors(buttons, doors, door_res);
    }
}
