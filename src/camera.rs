use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{despawn::Keep, schedule::InGameSet, ship::SpaceShip};

// const CAM_DISTANCE: f32 = 140.;
const CHASE_FACTOR: f32 = 1.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(
            PostUpdate,
            fallow_player
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        );
        // app.insert_resource(ClearColor(Color::rgb(0.1, 0., 0.15)));
        app.insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 750.,
        });
    }
}

fn fallow_player(
    mut q: Query<&mut Transform, With<Camera>>,
    q_player: Query<&Transform, (Without<Camera>, With<SpaceShip>)>,
    time: Res<Time>,
) {
    let player = q_player.get_single();
    let camera = q.get_single_mut();

    if let (Ok(mut camera), Ok(player)) = (camera, player) {
        let Vec3 { x, y, .. } = player.translation;
        let z = camera.translation.z;
        let target = Vec3 { x, y, z };
        let dt = time.delta_seconds();
        // camera.translation = camera.translation.lerp(target, dt * CHASE_FACTOR);
        camera.translation[0] = player.translation[0];
        camera.translation[1] = player.translation[1];
    }
}

fn spawn_camera(mut commands: Commands) {
    // let camera = Camera3dBundle {
    //     transform: Transform::from_xyz(0., 0., CAM_DISTANCE).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..Default::default()
    // };

    // let mut camera = Camera2dBundle::new_with_far(100.0);
    // camera.projection.scale = 0.1;
    // let camera = Camera2dBundle::new_with_far(120.);
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.1;
    commands.spawn((camera, Keep));
}
