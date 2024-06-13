use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

use crate::{collide::Collider, schedule::InGameSet};

pub struct MovePlug;
impl Plugin for MovePlug {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_velocity, update_position)
                .chain()
                .in_set(InGameSet::EntityUpdate),
        );
    }
}

#[derive(Component, Debug, Default)]
pub struct Acc(Vec3);

impl From<Vec3> for Acc {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Component, Debug, Default)]
pub struct Velocity(pub Vec3);

impl From<Vec3> for Velocity {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

impl Deref for Velocity {
    type Target = Vec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Velocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Bundle)]
pub struct MovingObj {
    pub velocity: Velocity,
    pub acc: Acc,
    pub model: SceneBundle,
    pub collider: Collider,
}

// impl From<SceneBundle> for MovingObj {
//     fn from(model: SceneBundle) -> Self {
//         Self {
//             model,
//             ..Default::default()
//         }
//     }
// }

fn update_velocity(mut q: Query<(&Acc, &mut Velocity)>, time: Res<Time>) {
    for (acc, mut velocity) in q.iter_mut() {
        velocity.0 += acc.0 * time.delta_seconds();
    }
}

fn update_position(mut q: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut position) in q.iter_mut() {
        position.translation += velocity.0 * time.delta_seconds();
    }
}
