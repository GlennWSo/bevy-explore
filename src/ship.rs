use bevy::prelude::*;

use crate::movement::{Acc, MovingObj, Velocity};

const START_TRANSLATION: Vec3 = Vec3::new(0., 0., -20.);
const START_VELOCITY: Vec3 = Vec3::new(0., 0., 1.);

pub struct ShipPlug;

fn spawn_spaceship(mut cmds: Commands, asset_server: Res<AssetServer>) {
    // let model_handel = asset_server.load("path/to/thing.glb#Scene0");

    let model = SceneBundle {
        scene: asset_server.load("Spaceship.glb#Scene0"),
        transform: Transform::from_translation(START_TRANSLATION),
        ..Default::default()
    };

    let ship = MovingObj {
        model,
        velocity: START_VELOCITY.into(),
        ..Default::default()
    };
    cmds.spawn(ship);
}

impl Plugin for ShipPlug {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_spaceship);
    }
}
