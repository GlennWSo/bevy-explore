use bevy::prelude::*;

const CAM_DISTANCE: f32 = 80.;

pub struct CameraPlugin;

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., CAM_DISTANCE, 0.).looking_at(Vec3::ZERO, Vec3::Z),
        ..Default::default()
    });
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}
