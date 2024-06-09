use bevy::prelude::*;
use learn_bevy::{movement::MovePlug, ship::ShipPlug, DebugPlugin};

fn main() {
    App::new()
        // .add_systems(Startup, spawn_spaceship)
        // .add_systems(Update, (update_position, print_position))
        .add_plugins(ShipPlug)
        .add_plugins(MovePlug)
        .add_plugins(DebugPlugin)
        .add_plugins(DefaultPlugins)
        .run();
}
