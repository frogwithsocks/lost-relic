use crate::{
    collide::{Collider, ColliderKind},
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
}

fn update_slider_collider(mut sliders: Query<(&mut Collider, &Slider, &mut Visibility)>) {
    for (mut collider, slider, mut visibility) in sliders.iter_mut() {
        visibility.is_visible = !slider.activated;
        if slider.activated {
            collider.kind = ColliderKind::None;
        } else {
            collider.kind = ColliderKind::Movable(f32::INFINITY);
        }
    }
}
