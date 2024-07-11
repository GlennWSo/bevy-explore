use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

use crate::assets::MyAssets;
use crate::schedule::InGameSet;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_dead.in_set(InGameSet::Despawn));
    }
}

fn despawn_dead(mut cmds: Commands, q: Query<(Entity, &Health)>, assets: Res<MyAssets>) {
    for (ent, Health { life, death_cry }) in q.iter() {
        if *life > 0 {
            continue;
        }
        if let DeathCry::Pop = death_cry {
            let sound = AudioBundle {
                source: assets.pop.clone(),
                settings: PlaybackSettings::DESPAWN,
            };
            cmds.spawn(sound);
        }
        cmds.entity(ent).despawn_recursive();
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum DeathCry {
    Pop,
    #[default]
    None,
}

#[derive(Component, Default, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct Health {
    pub life: i32,
    pub death_cry: DeathCry,
}

impl Deref for Health {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.life
    }
}

impl DerefMut for Health {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.life
    }
}
