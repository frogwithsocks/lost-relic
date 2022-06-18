use crate::{
    collide::{Collider},
    state::GameState,
};
use bevy::prelude::*;

pub struct SliderPlugin;

impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Play)
                .with_system(update_slider_collider)
                .before("collision"),
        );
    }
}

#[derive(Component, Default)]
pub struct Slider {
    pub activated: bool,
    pub extent: f32,
    pub max_compress: f32,
    pub change: f32,
}

fn update_slider_collider(
    mut sliders: Query<(&mut Transform, &mut Collider, &mut Sprite, &mut Slider)>,
) {
    for (mut transform, mut collider, mut sprite, mut slider) in sliders.iter_mut() {
        if slider.activated && slider.extent != slider.max_compress + slider.change {
            collider.size.y -= slider.change;
            sprite.custom_size = sprite
                .custom_size
                .map(|size| Vec2::new(size.x, (size.y - slider.change).max(0.0)));
            transform.translation.y -= slider.change / 2.0;
            slider.extent += slider.change;
        } else {
            if slider.extent != 0.0 {
                collider.size.y += slider.change;
                sprite.custom_size = sprite
                    .custom_size
                    .map(|size| Vec2::new(size.x, (size.y + slider.change).max(0.0)));
                transform.translation.y += slider.change / 2.0;
                slider.extent -= slider.change;
            }
        }
    }
}
