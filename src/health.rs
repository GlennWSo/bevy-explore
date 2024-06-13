use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use bevy::{audio::AudioPlugin, prelude::*};

use crate::assets::Assets;
use crate::schedule::InGameSet;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_dead.in_set(InGameSet::Despawn));
    }
}

fn despawn_dead(mut cmds: Commands, q: Query<(Entity, &Health)>, assets: Res<Assets>) {
    for (ent, Health { life, death_cry }) in q.iter() {
        if *life > 0 {
            continue;
        }
        match death_cry {
            DeathCry::Pop => {
                let sound = AudioBundle {
                    source: assets.pop.clone(),
                    settings: PlaybackSettings::DESPAWN,
                };
                cmds.spawn(sound);
            }
            _ => (),
        }
        cmds.entity(ent).despawn_recursive();
    }
}

#[derive(Default, Debug)]
pub enum DeathCry {
    Pop,
    #[default]
    None,
}

#[derive(Component, Default, Debug)]
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
