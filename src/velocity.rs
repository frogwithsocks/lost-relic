use bevy::prelude::*;

use crate::state::GameState;

pub struct VelocityPlugin;

impl Plugin for VelocityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Play).with_system(update_gravity));
    }
}

#[derive(Component, Debug)]
pub struct Gravity(f32);

impl Default for Gravity {
    fn default() -> Self {
        Gravity(100.0)
    }
}

fn update_gravity(mut query: Query<(&mut Velocity, &Gravity)>) {
    for (mut velocity, gravity) in query.iter_mut() {
        velocity.linvel.y -= gravity.0;
    }
}

#[derive(Component, Debug, Clone)]
pub struct Velocity {
    pub linvel: Vec3,
    pub drag: Vec3,
}

impl Default for Velocity {
    fn default() -> Self {
        Velocity {
            linvel: Vec3::ZERO,
            drag: Vec3::splat(0.95),
        }
    }
}
