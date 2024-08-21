use std::f32::consts::PI;
use std::marker::PhantomData;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::assets::MyAssets;
use crate::collide_dmg::CollisionDamage;
use crate::despawn::Keep;
use crate::guns::{GunFireEvent, NinjaGun, PlasmaGun};
use crate::health::Health;
use crate::schedule::InGameSet;
use crate::state::GameState;
use crate::Player;

const SHIP_HEALTH: i32 = 1000000;
const SHIP_COLLISION_DAMAGE: i32 = 30;

pub struct ShipPlug;

impl Plugin for ShipPlug {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_player_ship);
        app.add_systems(OnExit(GameState::GameOver), spawn_player_ship);
        app.add_systems(Update, shield_ctrl.in_set(InGameSet::UI));
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

impl SpaceShip {
    pub const FORWARD_OFFSET: f32 = 8.5;
}

#[derive(Component, Debug)]
struct Shield;

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
    pub entity: Entity,
    pub throttle: Vec2,
    pub steering: f32,
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

fn spawn_player_ship(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<MyAssets>,
) {
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
