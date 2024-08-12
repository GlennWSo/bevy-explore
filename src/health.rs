use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

use crate::assets::MyAssets;
use crate::schedule::InGameSet;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_dead, detect_dead)
                .chain()
                .in_set(InGameSet::Despawn),
        );
        app.add_event::<Death>();
    }
}

#[derive(Event, Deref)]
pub struct Death(Entity);

fn detect_dead(mut writer: EventWriter<Death>, q: Query<(Entity, &Health)>) {
    for (ent, Health { life, .. }) in q.iter() {
        if *life > 0 {
            continue;
        }
        writer.send(Death(ent));
    }
}

fn despawn_dead(mut cmds: Commands, mut reader: EventReader<Death>) {
    for e in reader.read() {
        cmds.entity(**e).despawn_recursive();
    }
}

pub trait DeathCry {
    fn cry(&self, assets: &MyAssets) -> AudioBundle;
}

pub fn cry_dead<T: Component + DeathCry>(
    mut cmds: Commands,
    mut reader: EventReader<Death>,
    assets: Res<MyAssets>,
    q: Query<&T>,
) {
    for e in reader.read() {
        let Ok(component) = q.get(**e) else {
            continue;
        };
        let sound = component.cry(&assets);
        cmds.spawn(sound);
    }
}

#[derive(Component, Default, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct Health {
    pub life: i32,
    // pub death_cry: DeathCry,
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
