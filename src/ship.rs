use bevy::prelude::*;

use crate::movement::Velocity;

pub struct ShipPlug;

fn spawn_spaceship(mut cmds: Commands, asset_server: Res<AssetServer>) {
    // let model_handel = asset_server.load("path/to/thing.glb#Scene0");
    cmds.spawn((
        SpatialBundle::default(),
        Velocity(Vec3 {
            x: 1.,
            ..Default::default()
        }),
    ));
}

impl Plugin for ShipPlug {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spaceship);
    }
}
