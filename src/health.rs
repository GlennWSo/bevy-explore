use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

use crate::schedule::InGameSet;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_dead.in_set(InGameSet::Despawn));
    }
}

fn despawn_dead(mut cmds: Commands, q: Query<(Entity, &Health)>) {
    for (ent, health) in q.iter() {
        if **health <= 0 {
            cmds.entity(ent).despawn_recursive();
        }
    }
}

#[derive(Component, Debug)]
pub struct Health(pub i32);

impl Deref for Health {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Health {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
