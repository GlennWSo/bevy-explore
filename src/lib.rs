#![feature(trait_alias)]
#![allow(clippy::type_complexity)]

pub mod assets;
pub mod astroids;
pub mod camera;
pub mod collide_dmg;
pub mod despawn;
pub mod guns;
pub mod health;
pub mod keyboard;
pub mod layers;
pub mod schedule;
pub mod sentry;
pub mod ship;
pub mod stage;
pub mod state;
pub mod ui;
pub mod zones;

use bevy::prelude::*;
use schedule::InGameSet;

pub struct DebugPlug;
#[derive(Component)]
pub struct Player;

fn print_position(q: Query<(Entity, &Transform)>) {
    for (entity, position) in q.iter() {
        info!("Enity {:?} is at position {:?}", entity, position)
    }
}
impl Plugin for DebugPlug {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, print_position.after(InGameSet::EntityUpdate));
    }
}
