use avian2d::prelude::*;
use bevy::prelude::*;
#[allow(unused)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use learn_bevy::{
    assets::AssetPlug, astroids::AstriodPlug, camera::CameraPlugin,
    collide_dmg::CollideDamagePlugin, despawn::DespawnPlugin, guns::GunPlugin,
    health::HealthPlugin, schedule::SchedulePlugin, sentry::SentryPlugin, ship::ShipPlug,
    state::StatePlugin, ui::UIPlugin, zones::ZonePlugin,
};

fn main() {
    let mut app = App::new();

    // #[cfg(target_arch = "wasm32")]
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            // provide the ID selector string here
            canvas: Some("#game".into()),
            // ... any other window properties ...
            ..default()
        }),
        ..default()
    }))
    .add_plugins(PhysicsPlugins::default().with_length_unit(1.))
    .insert_resource(Gravity(Vec2::ZERO));

    // #[cfg(not(target_arch = "wasm32"))]
    // app.add_plugins(WorldInspectorPlugin::new());

    // .add_plugins(DebugPlug)
    app.add_plugins(CollideDamagePlugin)
        .add_plugins(StatePlugin)
        .add_plugins(SchedulePlugin)
        .add_plugins(AssetPlug)
        .add_plugins(HealthPlugin)
        .add_plugins(ShipPlug)
        .add_plugins(AstriodPlug)
        .add_plugins(GunPlugin)
        .add_plugins(DespawnPlugin)
        .add_plugins(ZonePlugin)
        // .add_plugins(SentryPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(CameraPlugin);

    app.run();
}
