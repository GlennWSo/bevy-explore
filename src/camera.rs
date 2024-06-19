use bevy::prelude::*;

use crate::{despawn::Keep, movement::Velocity, schedule::InGameSet, ship::SpaceShip};

const CAM_DISTANCE: f32 = 140.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Update, fallow_player.after(InGameSet::EntityUpdate));
    }
}

fn fallow_player(
    mut q: Query<&mut Transform, With<Camera>>,
    q_player: Query<&Transform, (Without<Camera>, With<SpaceShip>)>,
) {
    let camera = q.get_single_mut();
    let player = q_player.get_single();

    if let (Ok(mut camera), Ok(player)) = (camera, player) {
        camera.translation[0] = player.translation[0];
        camera.translation[2] = player.translation[2];
    }
}

fn spawn_camera(mut commands: Commands) {
    let camera = Camera3dBundle {
        transform: Transform::from_xyz(0., CAM_DISTANCE, 0.).looking_at(Vec3::ZERO, Vec3::Z),
        ..Default::default()
    };
    let velocity = Velocity::default();
    commands.spawn((camera, Keep, velocity));
}
