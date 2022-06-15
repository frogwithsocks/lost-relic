use bevy::prelude::*;

pub struct VelocityPlugin;

impl Plugin for VelocityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_gravity.label("gravity").after("player"))
            .add_system(update_velocity.label("velocity").after("gravity"));
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

fn update_velocity(mut query: Query<(&mut Velocity, &mut Transform)>, time: Res<Time>) {
    for (mut velocity, mut transform) in query.iter_mut() {
        let drag = velocity.drag;
        transform.translation =
            (transform.translation + velocity.linvel * time.delta_seconds()).floor();
        velocity.linvel.x =
            velocity.linvel.x - (velocity.linvel.x * (drag.x * time.delta_seconds()));
        velocity.linvel.y =
            velocity.linvel.y - (velocity.linvel.y * (drag.y * time.delta_seconds()));
    }
}
