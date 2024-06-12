use bevy::prelude::*;
use learn_bevy::{
    assets::AssetPlug, astroids::AstriodPlug, camera::CameraPlugin, collide::CollidePlugin,
    despawn::DespawnPlugin, health::HealthPlugin, movement::MovePlug, schedule::SchedulePlugin,
    ship::ShipPlug, state::StatePlugin, DebugPlug,
};

fn main() {
    let mut app = App::new();

    // #[cfg(not(target_arch = "wasm32"))]
    // app.add_plugins(DefaultPlugins);

    // #[cfg(target_arch = "wasm32")]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            // provide the ID selector string here
            canvas: Some("#game".into()),
            // ... any other window properties ...
            ..default()
        }),
        ..default()
    }));

    app.insert_resource(ClearColor(Color::rgb(0.1, 0., 0.15)))
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 750.,
        })
        .add_plugins(StatePlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(AssetPlug)
        .add_plugins(HealthPlugin)
        .add_plugins(ShipPlug)
        .add_plugins(AstriodPlug)
        .add_plugins(MovePlug)
        // .add_plugins(DebugPlug)
        .add_plugins(DespawnPlugin)
        .add_plugins(CollidePlugin)
        .add_plugins(CameraPlugin);

    app.run();
}
