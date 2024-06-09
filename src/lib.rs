pub mod movement;
pub mod ship;

use bevy::prelude::*;

pub struct DebugPlugin;

fn print_position(q: Query<(Entity, &Transform)>) {
    for (entity, position) in q.iter() {
        info!("Enity {:?} is at position {:?}", entity, position)
    }
}
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, print_position);
    }
}
