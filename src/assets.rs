use bevy::{math::primitives::Circle, prelude::*};

use crate::schedule::InitStages;
// use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

#[derive(Resource, Default)]
pub struct MyAssets {
    pub astriod: Handle<Image>,
    pub ship: Handle<Scene>,
    pub missles: Handle<Scene>,
    pub pop: Handle<AudioSource>,
    pub laser_sound: Handle<AudioSource>,
    pub crack: Handle<AudioSource>,
    pub ball: Handle<Mesh>,
    pub asteroid_material: Handle<ColorMaterial>,
    pub doing: Handle<AudioSource>,
    pub slap: Handle<AudioSource>,
    pub muffled_laser: Handle<AudioSource>,
}

pub struct AssetPlug;
impl Plugin for AssetPlug {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyAssets>()
            .add_systems(Startup, setup.in_set(InitStages::LoadAssets));
    }
}

fn setup(
    mut assets: ResMut<MyAssets>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
) {
    let astroid_shape = Circle::new(1.0);
    let astroid_mesh = meshes.add(astroid_shape);

    let laser_color = Color::srgb(0., 1., 0.);
    let laser_color_handle = colors.add(laser_color);

    *assets = MyAssets {
        astriod: asset_server.load("baren.png"),
        // astriod: asset_server.load("Planet.glb#Scene0"),
        ship: asset_server.load("Spaceship.glb#Scene0"),
        missles: asset_server.load("BulletsPickup.glb#Scene0"),
        pop: asset_server.load("ball_tap2073.wav"),
        laser_sound: asset_server.load("laser-104024.mp3"),
        ball: astroid_mesh,
        asteroid_material: laser_color_handle,
        crack: asset_server.load("524610__clearwavsound__fruit-crack.wav"),
        doing: asset_server.load("funny_boing_1_miksmusic.wav"),
        slap: asset_server.load("glass_slapp1_cjspellsfish.wav"),
        muffled_laser: asset_server.load("muffled_laser_blast_samsterbirdies.mp3"),
    }
}
