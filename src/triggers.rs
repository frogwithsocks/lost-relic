use std::collections::HashMap;
use bevy::prelude::*;
use crate::collide::GameEvent;

pub struct DoorRes(HashMap<String, usize>);
pub struct ButtonRes(HashMap<u32, Button>);

pub struct Button {
    pressed: bool,
    pub door: String,
}

impl Button {
    fn is_pressed(&self) -> bool {
        self.pressed
    }

    fn toggle(&mut self) {
        self.pressed = !self.pressed;
    }
}