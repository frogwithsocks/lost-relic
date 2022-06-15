use std::collections::HashMap;

pub struct DoorRes(pub HashMap<String, usize>);
pub struct ButtonRes(pub HashMap<u32, Button>);

pub struct Button {
    pressed: bool,
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