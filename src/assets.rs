
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Assets {
    pub astriod: Handle<Scene>,
    pub ship: Handle<Scene>,
    pub missles: Handle<Scene>,
    pub pop: Handle<AudioSource>,
    pub laser: Handle<AudioSource>,
}

pub struct AssetPlug;

fn load_assets(mut assets: ResMut<Assets>, asset_server: Res<AssetServer>) {
    *assets = Assets {
        astriod: asset_server.load("Planet.glb#Scene0"),
        ship: asset_server.load("Spaceship.glb#Scene0"),
        missles: asset_server.load("BulletsPickup.glb#Scene0"),
        pop: asset_server.load("ball_tap2073.wav"),
        laser: asset_server.load("laser-104024.mp3"),
    }
}

impl Plugin for AssetPlug {
    fn build(&self, app: &mut App) {
        app.init_resource::<Assets>()
            .add_systems(Startup, load_assets);
    }
}
