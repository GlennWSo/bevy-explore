use bevy::audio::Volume;
use bevy::prelude::*;
// use bevy::input::InputSystem

use crate::assets::Assets;
use crate::collide::{Collider, CollisionDamage};
use crate::despawn::{despawn_far, Keep};
use crate::health::{DeathCry, Health};
use crate::movement::{MovingObj, Velocity};
use crate::schedule::InGameSet;
use crate::state::GameState;

// const START_TRANSLATION: Vec3 = Vec3::new(0., 0., -20.);
const SHIP_SPEED: f32 = 25.0;
const SHIP_ROTATION_SPEED: f32 = 2.5;
const SHIP_ROLL_SPEED: f32 = 2.5;
const SHIP_HEALTH: i32 = 1000000;
const SHIP_COLLISION_DAMAGE: i32 = 30;

pub struct ShipPlug;

impl Plugin for ShipPlug {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_spaceship);
        app.add_systems(OnExit(GameState::GameOver), spawn_spaceship);
        app.add_systems(
            Update,
            (ship_movement_ctrl, ship_weapon_ctrl, shield_ctrl)
                .chain()
                .in_set(InGameSet::UI),
        )
        .add_systems(Update, despawn_far::<Missle, 1000>)
        .add_systems(Update, end_player);
    }
}

#[derive(Component)]
pub struct SpaceShip;

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug)]
struct Shield;

#[derive(Component)]
pub struct Missle;

impl Missle {
    const SPEED: f32 = 50.0;
    const FORWARD_OFFSET: f32 = 7.5;
    const HEALTH: i32 = 1;
    const DAMAGE: i32 = 10;
}
#[derive(Component)]
struct MissleLauncher {
    cooldown: f32,
    ready: WeponState,
}

impl MissleLauncher {
    const START_RATE: f32 = 0.02;
    /// fire rate from seconds interval
    fn new(cooldown: f32) -> Self {
        assert!(cooldown >= 0.0);
        Self {
            cooldown,
            ready: WeponState::Ready,
        }
    }
    fn fire(&mut self) {
        self.ready = WeponState::Cooling(self.cooldown);
    }
    fn update(&mut self, dt: f32) {
        match self.ready {
            WeponState::Ready => {}
            WeponState::Cooling(mut time_left) => {
                time_left -= dt;
                if time_left < 0. {
                    self.ready = WeponState::Ready;
                } else {
                    self.ready = WeponState::Cooling(time_left)
                }
            }
        }
    }
}

impl Default for MissleLauncher {
    fn default() -> Self {
        Self::new(Self::START_RATE)
    }
}

fn shield_ctrl(
    mut cmds: Commands,
    q: Query<Entity, With<SpaceShip>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(ship) = q.get_single() else {
        return;
    };

    if input.pressed(KeyCode::Tab) {
        cmds.entity(ship).insert(Shield);
    }
}

// type ShipQuery = Query<(&mut Transform, &mut Velocity), With<SpaceShip>>;
fn ship_movement_ctrl(
    mut q: Query<(&mut Transform, &mut Velocity), With<SpaceShip>>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut velocity)) = q.get_single_mut() else {
        return;
    };

    let mut movement = 0.0;
    if key_input.pressed(KeyCode::ArrowDown) {
        movement = -SHIP_SPEED;
    } else if key_input.pressed(KeyCode::ArrowUp) {
        movement = SHIP_SPEED;
    }

    let mut rotation = 0.0;
    if key_input.pressed(KeyCode::ArrowLeft) {
        rotation = SHIP_ROTATION_SPEED;
    } else if key_input.pressed(KeyCode::ArrowRight) {
        rotation = -SHIP_ROLL_SPEED;
    }

    let mut roll = 0.0;
    if key_input.pressed(KeyCode::KeyA) {
        roll = SHIP_ROLL_SPEED;
    } else if key_input.pressed(KeyCode::KeyD) {
        roll = -SHIP_ROLL_SPEED;
    }

    let dt = time.delta_seconds();
    transform.rotate_z(rotation * dt);
    transform.rotate_local_y(-roll * dt);

    velocity.0 += -transform.forward().truncate() * movement * dt;
}

fn spawn_spaceship(mut cmds: Commands, assets: Res<Assets>) {
    // let model_handel = asset_server.load("path/to/thing.glb#Scene0");

    let mut transform = Transform::default();
    transform.rotate_x(90.0f32.to_radians());
    let model = SceneBundle {
        scene: assets.ship.clone(),
        transform,
        ..Default::default()
    };

    let collider = Collider::new(4.0);
    let obj = MovingObj {
        model,
        velocity: Vec2::ZERO.into(),
        acc: Vec2::ZERO.into(),
    };
    let ship = (
        obj,
        collider,
        Player,
        SpaceShip,
        MissleLauncher::new(0.05),
        Keep,
        Health {
            life: SHIP_HEALTH,
            ..Default::default()
        },
        CollisionDamage(SHIP_COLLISION_DAMAGE),
        Name::new("PlayerShip"),
    );
    cmds.spawn(ship);
}

enum WeponState {
    Ready,
    Cooling(f32),
}

fn ship_weapon_ctrl(
    mut cmds: Commands,
    mut q: Query<(&Transform, &Velocity, &mut MissleLauncher), With<SpaceShip>>,
    btn_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    assets: Res<Assets>,
) {
    if !btn_input.pressed(KeyCode::Space) {
        return;
    }
    // println!("Fire at will!");

    let Ok((ship_transform, ship_velocity, mut missle_launcher)) = q.get_single_mut() else {
        return;
    };

    match missle_launcher.ready {
        WeponState::Ready => missle_launcher.fire(),
        WeponState::Cooling(_time_remaining) => {
            let dt = time.delta_seconds();
            missle_launcher.update(dt);
            return;
        }
    }

    let mut transform = *ship_transform;
    let velocity = (-transform.forward().truncate() * Missle::SPEED + **ship_velocity).into();
    transform.translation -= Missle::FORWARD_OFFSET * *ship_transform.forward();
    transform.rotate_local_y(90.0_f32.to_radians());

    let scene = assets.missles.clone();
    let model = SceneBundle {
        scene,
        transform,
        ..Default::default()
    };
    let settings = PlaybackSettings {
        mode: bevy::audio::PlaybackMode::Despawn,
        speed: 1.5,
        volume: Volume::new(0.3),
        ..Default::default()
    };
    let pew_sound = AudioBundle {
        source: assets.laser.clone(),
        settings,
    };
    // audio

    cmds.spawn(pew_sound);
    // let death_cry = Some(assets.pop.clone());
    // assets.pop.as_any()
    let moving_obj = MovingObj {
        model,
        velocity,
        acc: Vec2::ZERO.into(),
    };
    let missle = (
        moving_obj,
        Collider::new(0.1),
        Missle,
        Health {
            life: Missle::HEALTH,
            death_cry: DeathCry::Pop,
        },
        CollisionDamage(Missle::DAMAGE),
    );
    cmds.spawn(missle);
}

fn end_player(mut next: ResMut<NextState<GameState>>, q: Query<(), With<SpaceShip>>) {
    if q.get_single().is_err() {
        next.set(GameState::GameOver)
    }
}
