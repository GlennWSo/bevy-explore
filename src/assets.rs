use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct Assets {
    pub astriod: Handle<Scene>,
    pub ship: Handle<Scene>,
    pub missles: Handle<Scene>,
}

pub struct AssetPlug;

fn load_assets(mut assets: ResMut<Assets>, asset_server: Res<AssetServer>) {
    *assets = Assets {
        astriod: asset_server.load("Planet.glb#Scene0"),
        ship: asset_server.load("Spaceship.glb#Scene0"),
        missles: asset_server.load("Bullets Pickup.glb#Scene0"),
    }
}

impl Plugin for AssetPlug {
    fn build(&self, app: &mut App) {
        app.init_resource::<Assets>()
            .add_systems(Startup, load_assets);
    }
}
