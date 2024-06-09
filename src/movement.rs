use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Acc(Vec3);

impl From<Vec3> for Acc {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

#[derive(Component, Debug, Default)]
pub struct Velocity(pub Vec3);

impl From<Vec3> for Velocity {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

#[derive(Bundle, Default)]
pub struct MovingObj {
    pub velocity: Velocity,
    pub acc: Acc,
    pub model: SceneBundle,
}

impl From<SceneBundle> for MovingObj {
    fn from(model: SceneBundle) -> Self {
        Self {
            model,
            ..Default::default()
        }
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
