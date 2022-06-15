use std::collections::HashMap;

pub struct DoorRes(pub HashMap<String, usize>);
pub struct ButtonRes(pub HashMap<u32, Button>);

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