use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Velocity(pub Vec3);

pub struct MovePlug;

fn update_position(mut q: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut position) in q.iter_mut() {
        position.translation += velocity.0;
    }
}

impl Plugin for MovePlug {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_position);
    }
}
