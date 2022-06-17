use bevy::prelude::*;
use crate::{
    collide::{
        Collider,
        ColliderKind,
        TOP,
    },
    velocity::Velocity,
    map::BLOCK_SIZE,
    state::GameState,
};

pub struct SliderPlugin;

impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_enter(GameState::Play)
                    .with_system(test_slider)
            )
            .add_system_set(
                SystemSet::on_update(GameState::Play)
                    .with_system(update_slider_collider)
                    .before("collision")
            );
    }
}

fn test_slider(mut commands: Commands) {
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
            color: Color::BLUE,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(2.5, 3.5, 2.0) * BLOCK_SIZE,
            ..default()
        },
        ..default()
    })
    .insert(Collider {
        kind: ColliderKind::Movable(10.0),
        size: Vec2::new(BLOCK_SIZE, BLOCK_SIZE),
        ..default()
    })
    .insert(Slider {
        activated: false,
        change: 1.0,
        max_extent: 0.0/*BLOCK_SIZE / 2.0*/,
        extent: 0.0,
    });
}

#[derive(Component)]
pub struct Slider {
    pub activated: bool,
    extent: f32,
    max_extent: f32,
    change: f32,
}

fn update_slider_collider(mut sliders: Query<(&mut Transform, &mut Collider, &mut Sprite, &mut Slider)>) {
    for (mut transform, mut collider, mut sprite, mut slider) in sliders.iter_mut() {
        slider.activated = collider.flags & TOP != 0;
        if slider.activated && slider.extent != slider.max_extent {
            collider.size.y -= slider.change;
            sprite.custom_size = sprite.custom_size.map(|size| Vec2::new(size.x, (size.y - slider.change).max(0.0)));
            transform.translation.y -= slider.change / 2.0;
            slider.extent += slider.change;
        } else {
            if slider.extent != 0.0 {
                collider.size.y += slider.change;
                sprite.custom_size = sprite.custom_size.map(|size| Vec2::new(size.x, (size.y + slider.change).max(0.0)));
                transform.translation.y += slider.change / 2.0;
                slider.extent -= slider.change; 
            }
        }
    }
}