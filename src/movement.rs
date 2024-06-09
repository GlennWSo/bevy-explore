use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Velocity(pub Vec3);

impl From<Vec3> for Velocity {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

pub struct MovePlug;

fn update_position(mut q: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut position) in q.iter_mut() {
        position.translation += velocity.0 * time.delta_seconds();
    }
}

impl Plugin for MovePlug {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_position);
    }
}
