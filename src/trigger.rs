use bevy::prelude::*;
use std::collections::HashMap;

pub struct DoorRes(pub HashMap<String, (usize, Entity)>);

#[derive(Component)]
pub struct Button {
    pub pressed: bool,
    pub door: String,
}

impl Button {
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn toggle(&mut self) {
        self.pressed = !self.pressed;
    }
}
