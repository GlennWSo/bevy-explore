use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

use crate::{collide::Collider, schedule::InGameSet};

pub struct MovePlug;
impl Plugin for MovePlug {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>().add_systems(
            Update,
            (update_velocity, update_position)
                .chain()
                .in_set(InGameSet::EntityUpdate),
        );
    }
}

#[derive(Clone, Component, Debug, Default, Deref, DerefMut)]
pub struct Acc(Vec2);

impl From<Vec2> for Acc {
    fn from(value: Vec2) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Component, Debug, Default, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct Velocity(pub Vec2);

impl From<Vec2> for Velocity {
    fn from(value: Vec2) -> Self {
        Self(value)
    }
}

#[derive(Bundle, Default)]
pub struct MovingObj {
    pub velocity: Velocity,
    pub acc: Acc,
    pub model: SceneBundle,
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
    for (velocity, mut transolation) in q.iter_mut() {
        let delta = time.delta_seconds() * **velocity;
        transolation.translation += delta.extend(0.0);
    }
}
