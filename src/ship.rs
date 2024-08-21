use std::f32::consts::PI;
use std::marker::PhantomData;

use avian2d::parry::utils::hashmap::FxHasher32;
use avian2d::prelude::*;
use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use rand::thread_rng;
// use bevy::input::InputSystem

use crate::assets::MyAssets;
use crate::collide_dmg::CollisionDamage;
use crate::despawn::Keep;
use crate::guns::{GunFireEvent, NinjaGun, PlasmaGun};
use crate::health::Health;
use crate::schedule::InGameSet;
use crate::state::GameState;

// const START_TRANSLATION: Vec3 = Vec3::new(0., 0., -20.);
const SHIP_SPEED: f32 = 25.0;
const SHIP_ROTATION_SPEED: f32 = 2.5;
const SHIP_ROLL_SPEED: f32 = 2.5;
const SHIP_HEALTH: i32 = 1000000;
const SHIP_COLLISION_DAMAGE: i32 = 30;

const FORWARD_OFFSET: f32 = 8.5;
pub struct ShipPlug;

impl Plugin for ShipPlug {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_player_ship);
        app.add_systems(OnExit(GameState::GameOver), spawn_player_ship);
        app.add_systems(Update, ship_weapon_ctrl.in_set(InGameSet::UI));
        app.add_systems(
            Update,
            (ship_movement_ctrl, shield_ctrl).in_set(InGameSet::UI),
        );
        app.add_systems(Update, ship_manuver.in_set(InGameSet::EntityUpdate))
            .add_event::<ManuverEvent>()
            .add_systems(Update, end_player);
    }
}

struct Mobility {
    /// m/s^2
    forward: f32,
    /// m/s^2
    reverse: f32,
    /// m/s^2
    strafe: f32,
    /// rad/s
    rotation: f32,
}

impl Mobility {
    /// calculate acceleration given throttle input
    fn accelerate(&self, throttle: &Vec2) -> Vec2 {
        let x = self.strafe * throttle.x;
        let factor = if throttle.y.is_sign_positive() {
            self.forward
        } else {
            self.reverse
        };
        let y = factor * throttle.y;
        Vec2 { x, y }
    }
}

impl Default for Mobility {
    fn default() -> Self {
        let forward = 25.0;
        let reverse = 10.0;
        let strafe = 15.0;
        let rotation = 2.5;
        Mobility {
            forward,
            reverse,
            strafe,
            rotation,
        }
    }
}

#[derive(Component, Default)]
pub struct SpaceShip {
    mobility: Mobility,
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug)]
struct Shield;

fn ship_weapon_ctrl(
    q: Query<(Entity, &Transform), With<Player>>,
    mut plasma_events: EventWriter<GunFireEvent<PlasmaGun>>,
    mut hook_events: EventWriter<GunFireEvent<NinjaGun>>,

    btn_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok((entity, ship_transform)) = q.get_single() else {
        return;
    };
    let translation = ship_transform.translation - *ship_transform.up() * FORWARD_OFFSET;
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
    if btn_input.pressed(KeyCode::ControlLeft) {
        hook_events.send(GunFireEvent {
            entity,
            transform: origin,
            phantom: PhantomData,
        });
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

#[derive(Event)]
pub struct ManuverEvent {
    entity: Entity,
    throttle: Vec2,
    steering: f32,
}

fn ship_manuver(
    mut q: Query<(&mut SpaceShip, &mut LinearVelocity, &mut Transform)>,
    mut reader: EventReader<ManuverEvent>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for ManuverEvent {
        entity,
        throttle,
        steering,
    } in reader.read()
    {
        let Ok((ship, mut velocity, mut transform)) = q.get_mut(*entity) else {
            continue;
        };
        let local_diff = dt * -ship.mobility.accelerate(throttle);
        let up = transform.up().truncate();
        let right: Vec2 = [up.y, -up.x].into();
        let y_diff = local_diff.y * up;
        let x_diff = local_diff.x * right;
        velocity.0 += x_diff + y_diff;
        transform.rotate_z(*steering * dt * -ship.mobility.rotation);
    }
}

// type ShipQuery = Query<(&mut Transform, &mut Velocity), With<SpaceShip>>;
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
    if key_input.pressed(KeyCode::KeyQ) {
        strafe = -1.0;
    } else if key_input.pressed(KeyCode::KeyE) {
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

fn spawn_player_ship(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<MyAssets>,
) {
    // transform.rotate_local_x(90.0f32.to_radians());
    // transform.rotate_x(90.0f32.to_radians());
    let mut shape = Triangle2d::default();
    shape.vertices.iter_mut().for_each(|v| {
        v.y = -v.y;
    });
    let mut scale = [1., 1.5, 1.0_f32].into();
    scale *= 5.0;
    let mut transform = Transform::from_xyz(0., 10., 0.).with_scale(scale);
    transform.rotate_z(PI);
    let model2d = SpriteBundle {
        transform,
        texture: assets.ship.clone(),
        sprite: Sprite {
            flip_y: true,
            custom_size: Some(Vec2 { x: 3., y: 3. }),
            ..default()
        },
        ..default()
    };

    let collider: Collider = shape.into();
    let mut camera = Camera2dBundle::default();
    camera.transform.rotate_z(180.0_f32.to_radians());
    camera.projection.scale = 0.1;
    let ship = (
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        model2d,
        collider,
        Player,
        SpaceShip::default(),
        PlasmaGun::new(0.15),
        NinjaGun::default(),
        Keep,
        Health {
            life: SHIP_HEALTH,
            ..Default::default()
        },
        CollisionDamage(SHIP_COLLISION_DAMAGE),
        Name::new("PlayerShip"),
    );
    let _entity1 = cmds.spawn(ship).id();

    // let entity2 = cmds
    //     .spawn((camera, Keep, RigidBody::Dynamic, Collider::circle(0.1)))
    //     .id();
    // let joint = FixedJoint::new(entity1, entity2);
    // cmds.spawn(joint);
}

// struct WeponState(f32);

fn end_player(mut next: ResMut<NextState<GameState>>, q: Query<(), With<SpaceShip>>) {
    if q.get_single().is_err() {
        next.set(GameState::GameOver)
    }
}
