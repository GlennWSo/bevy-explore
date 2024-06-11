pub mod assets;
pub mod astroids;
pub mod camera;
pub mod collide;
pub mod far;
pub mod movement;
pub mod schedule;
pub mod ship;
pub mod state;

use bevy::prelude::*;
use schedule::InGameSet;

pub struct DebugPlug;

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
