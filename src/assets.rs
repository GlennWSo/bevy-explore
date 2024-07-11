use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

#[derive(Resource, Default)]
pub struct MyAssets {
    pub astriod: Handle<Scene>,
    pub ship: Handle<Scene>,
    pub missles: Handle<Scene>,
    pub pop: Handle<AudioSource>,
    pub laser_sound: Handle<AudioSource>,
    pub laser: Mesh2dHandle,
    pub laser_color: Handle<ColorMaterial>,
}

pub struct AssetPlug;

fn setup(
    mut assets: ResMut<MyAssets>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
) {
    let shape = Capsule2d::new(5., 50.0);
    let handle = meshes.add(shape);

    let laser_color = Color::rgb(0., 1., 0.);
    let laser_color_handle = colors.add(laser_color);

    *assets = MyAssets {
        astriod: asset_server.load("Planet.glb#Scene0"),
        ship: asset_server.load("Spaceship.glb#Scene0"),
        missles: asset_server.load("BulletsPickup.glb#Scene0"),
        pop: asset_server.load("ball_tap2073.wav"),
        laser_sound: asset_server.load("laser-104024.mp3"),
        laser: Mesh2dHandle(handle),
        laser_color: laser_color_handle,
    }
}

impl Plugin for AssetPlug {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyAssets>().add_systems(Startup, setup);
    }
}
