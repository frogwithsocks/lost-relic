use bevy::prelude::*;

pub struct VelocityPlugin;

impl Plugin for VelocityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_velocity.label("velocity").after("player"));
    }
}

#[derive(Component, Default, Debug)]
pub struct Velocity {
    pub linvel: Vec3,
    pub drag: Vec3,
}

fn update_velocity(mut query: Query<(&mut Velocity, &mut Transform)>, time: Res<Time>) {
    for (mut velocity, mut transform) in query.iter_mut() {
        let drag = velocity.drag;
        transform.translation += velocity.linvel * time.delta_seconds();
        velocity.linvel.x = velocity.linvel.x - (velocity.linvel.x * (drag.x * time.delta_seconds()));
        velocity.linvel.y = velocity.linvel.y - (velocity.linvel.y * (drag.y * time.delta_seconds()));
    }
}
