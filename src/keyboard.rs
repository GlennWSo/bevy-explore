use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    guns::{GunFireEvent, NinjaGun, PlasmaGun, ReleaseHookEvent},
    schedule::InGameSet,
    ship::{ManuverEvent, SpaceShip},
    Player,
};

pub struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                fire_ctrl_hook,
                fire_ctrl_plasma,
                ship_movement_ctrl,
                ui_release_hook,
            )
                .in_set(InGameSet::UI),
        );
    }
}

fn ui_release_hook(
    mut writer: EventWriter<ReleaseHookEvent>,
    btn_input: Res<ButtonInput<KeyCode>>,
    q: Query<Entity, (With<Player>, With<NinjaGun>)>,
) {
    if !btn_input.pressed(KeyCode::Tab) {
        return;
    }

    let Ok(gun) = q.get_single() else {
        return;
    };
    writer.send(ReleaseHookEvent { gun });
}

fn fire_ctrl_plasma(
    q: Query<(Entity, &Transform), (With<Player>, With<SpaceShip>)>,
    mut plasma_events: EventWriter<GunFireEvent<PlasmaGun>>,
    btn_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok((entity, ship_transform)) = q.get_single() else {
        return;
    };
    let translation = ship_transform.translation - *ship_transform.up() * SpaceShip::FORWARD_OFFSET;
    let mut origin = ship_transform.clone();
    origin.scale = [1., 1., 1.].into();
    origin.translation = translation;

    if btn_input.pressed(KeyCode::Space) {
        plasma_events.send(GunFireEvent {
            entity,
            transform: origin,
            phantom: PhantomData,
        });
    }
}
fn fire_ctrl_hook(
    q: Query<(Entity, &Transform), (With<Player>, With<SpaceShip>)>,
    mut hook_events: EventWriter<GunFireEvent<NinjaGun>>,

    btn_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok((entity, ship_transform)) = q.get_single() else {
        return;
    };
    let translation = ship_transform.translation - *ship_transform.up() * SpaceShip::FORWARD_OFFSET;
    let mut origin = ship_transform.clone();
    origin.scale = [1., 1., 1.].into();
    origin.translation = translation;

    if btn_input.pressed(KeyCode::ControlLeft) {
        hook_events.send(GunFireEvent {
            entity,
            transform: origin,
            phantom: PhantomData,
        });
    }
}
fn ship_movement_ctrl(
    mut q: Query<Entity, (With<SpaceShip>, With<Player>)>,
    mut reporter: EventWriter<ManuverEvent>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(ship) = q.get_single_mut() else {
        return;
    };

    let mut forward = 0.0;
    if key_input.pressed(KeyCode::ArrowDown) {
        forward = -1.0;
    } else if key_input.pressed(KeyCode::ArrowUp) {
        forward = 1.0;
    }

    let mut strafe = 0.0;
    if key_input.pressed(KeyCode::KeyA) {
        strafe = -1.0;
    } else if key_input.pressed(KeyCode::KeyD) {
        strafe = 1.0;
    }

    let mut steering = 0.0;
    // steer left
    if key_input.pressed(KeyCode::ArrowLeft) {
        steering = -1.0;
    } else if key_input.pressed(KeyCode::ArrowRight) {
        steering = 1.0;
    }

    let any_input = (strafe != 0.0) || (forward != 0.0) || (steering != 0.0);
    if any_input {
        let throttle = Vec2 {
            y: forward,
            x: strafe,
        };
        reporter.send(ManuverEvent {
            entity: ship,
            throttle,
            steering,
        });
    }
}
