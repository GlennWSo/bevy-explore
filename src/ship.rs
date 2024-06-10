use bevy::prelude::*;
// use bevy::input::InputSystem

use crate::assets::Assets;
use crate::movement::{MovingObj, Velocity};

const START_TRANSLATION: Vec3 = Vec3::new(0., 0., -20.);
const SHIP_SPEED: f32 = 25.0;
const SHIP_ROTATION_SPEED: f32 = 2.5;
const SHIP_ROLL_SPEED: f32 = 2.5;

#[derive(Component)]
pub struct SpaceShip;
pub struct ShipPlug;

// type ShipQuery = Query<(&mut Transform, &mut Velocity), With<SpaceShip>>;
fn ship_movement_ctrl(
    mut q: Query<(&mut Transform, &mut Velocity), With<SpaceShip>>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (mut transform, mut velocity) = q.single_mut();

    let mut movement = 0.0;
    if key_input.pressed(KeyCode::ArrowDown) {
        movement = -SHIP_SPEED;
    } else if key_input.pressed(KeyCode::ArrowUp) {
        movement = SHIP_SPEED;
    }

    let mut rotation = 0.0;
    if key_input.pressed(KeyCode::ArrowLeft) {
        rotation = SHIP_ROTATION_SPEED;
    } else if key_input.pressed(KeyCode::ArrowRight) {
        rotation = -SHIP_ROLL_SPEED;
    }

    let mut roll = 0.0;
    if key_input.pressed(KeyCode::KeyA) {
        roll = SHIP_ROLL_SPEED;
    } else if key_input.pressed(KeyCode::KeyD) {
        roll = -SHIP_ROLL_SPEED;
    }

    transform.rotate_y(rotation * time.delta_seconds());
    transform.rotate_local_z(-roll * time.delta_seconds());

    velocity.0 = -transform.forward() * movement;
}

fn spawn_spaceship(mut cmds: Commands, assets: Res<Assets>) {
    // let model_handel = asset_server.load("path/to/thing.glb#Scene0");

    let model = SceneBundle {
        scene: assets.ship.clone(),
        transform: Transform::from_translation(START_TRANSLATION),
        ..Default::default()
    };

    let ship = MovingObj {
        model,
        ..Default::default()
    };
    cmds.spawn((ship, SpaceShip));
}

impl Plugin for ShipPlug {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_spaceship);
        app.add_systems(Update, ship_movement_ctrl);
    }
}
