use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
// use bevy::input::InputSystem

use crate::assets::MyAssets;
use crate::collide_dmg::CollisionDamage;
use crate::despawn::Keep;
use crate::guns::{NinjaGun, PlasmaGun};
use crate::health::Health;
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
            (ship_movement_ctrl, shield_ctrl).in_set(InGameSet::UI),
        )
        .add_systems(Update, end_player);
    }
}

#[derive(Component)]
pub struct SpaceShip;

#[derive(Component)]
pub struct Player;

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

// type ShipQuery = Query<(&mut Transform, &mut Velocity), With<SpaceShip>>;
fn ship_movement_ctrl(
    mut q: Query<(&mut Transform, &mut LinearVelocity), With<SpaceShip>>,
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

    velocity.0 += -(transform.up()).truncate() * movement * dt;
}

fn spawn_spaceship(
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
    let color: Color = css::BLUE.into();
    // let model2d = MaterialMesh2dBundle {
    //     mesh: meshes.add(shape).into(),
    //     transform,
    //     material: materials.add(color),
    //     ..default()
    // };
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

    // let derp = HomeMadeCollider::new(4.0);
    let collider: Collider = shape.into();
    let mut camera = Camera2dBundle::default();
    camera.transform.rotate_z(180.0_f32.to_radians());
    camera.projection.scale = 0.1;
    let ship = (
        // Velocity::default(),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        model2d,
        collider,
        // derp,
        Player,
        SpaceShip,
        PlasmaGun::new(0.07),
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
