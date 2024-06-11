use bevy::prelude::*;
use learn_bevy::{
    assets::AssetPlug, astroids::AstriodPlug, camera::CameraPlugin, collide::CollidePlugin,
    far::FarPlugin, movement::MovePlug, schedule::SchedulePlugin, ship::ShipPlug,
    state::StatePlugin, DebugPlug,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0., 0.15)))
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 750.,
        })
        .add_plugins(StatePlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(AssetPlug)
        .add_plugins(ShipPlug)
        .add_plugins(AstriodPlug)
        .add_plugins(MovePlug)
        // .add_plugins(DebugPlug)
        .add_plugins(FarPlugin)
        .add_plugins(CollidePlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(DefaultPlugins)
        .run();
}
