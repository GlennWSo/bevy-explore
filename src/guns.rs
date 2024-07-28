use avian2d::prelude::*;
use bevy::{audio::Volume, prelude::*, sprite::MaterialMesh2dBundle};

use crate::collide_dmg::CollisionDamage;
use crate::{
    assets::MyAssets,
    despawn::despawn_far,
    health::{DeathCry, Health},
    schedule::InGameSet,
    ship::Player,
};

const FORWARD_OFFSET: f32 = 7.5;

pub struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            cooldown_guns::<PlasmaGun>.in_set(InGameSet::EntityUpdate),
        )
        .add_systems(Update, ship_weapon_ctrl.in_set(InGameSet::UI))
        .add_systems(Update, despawn_far::<Plasma, 10_000>);
    }
}

#[derive(Component)]
pub struct NinjaHook;

impl NinjaHook {
    const SPEED: f32 = 80.0;
}

#[derive(Component)]
pub struct Plasma;

impl Plasma {
    const SPEED: f32 = 80.0;
    const DAMAGE: i32 = 10;
    const DENSITY: f32 = 5.0;
}

#[derive(Component)]
pub struct PlasmaGun {
    fire_interval: f32,
    count_down: f32,
}

pub trait Gun {
    type Missle: Component;
    /// dt time since frame
    fn fire(&mut self) -> Option<Self::Missle>;
    fn cooldown(&mut self, dt: f32);
}

impl Default for PlasmaGun {
    fn default() -> Self {
        Self::new(Self::DEFAULT_RATE)
    }
}

impl PlasmaGun {
    const DEFAULT_RATE: f32 = 0.1;
    /// fire rate from seconds interval
    pub fn new(cooldown: f32) -> Self {
        assert!(cooldown >= 0.0);
        Self {
            fire_interval: cooldown,
            count_down: 0.0,
        }
    }
}

impl Gun for PlasmaGun {
    type Missle = Plasma;
    fn fire(&mut self) -> Option<Plasma> {
        if self.count_down <= 0.0 {
            self.count_down = self.fire_interval;
            Some(Plasma)
        } else {
            None
        }
    }

    fn cooldown(&mut self, dt: f32) {
        if self.count_down <= 0.0 {
            return;
        }
        self.count_down -= dt;
    }
}

fn ship_weapon_ctrl(
    mut cmds: Commands,
    mut q: Query<(&Transform, &LinearVelocity, &mut PlasmaGun), With<Player>>,
    btn_input: Res<ButtonInput<KeyCode>>,
    // time: Res<Time>,
    assets: Res<MyAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !btn_input.pressed(KeyCode::Space) {
        return;
    }
    // println!("Fire at will!");

    let Ok((ship_transform, ship_velocity, mut plasma_gun)) = q.get_single_mut() else {
        return;
    };

    // let derp = Missle;
    fire_plasma(
        plasma_gun,
        ship_transform,
        ship_velocity,
        &mut materials,
        &mut meshes,
        &assets,
        &mut cmds,
    );
}

fn fire_plasma(
    mut plasma_gun: Mut<PlasmaGun>,
    ship_transform: &Transform,
    ship_velocity: &LinearVelocity,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    assets: &Res<MyAssets>,
    cmds: &mut Commands,
) {
    let Some(plasma) = plasma_gun.fire() else {
        return;
    };

    let mut transform = ship_transform.with_scale(Vec3::ONE);
    let velocity: LinearVelocity =
        (-transform.up().truncate() * Plasma::SPEED + **ship_velocity).into();
    transform.translation -= FORWARD_OFFSET * *ship_transform.up();

    let shape = Capsule2d::new(0.5, 2.);
    let collider = Collider::capsule(0.5, 0.2);
    let color = Color::srgb(7.5, 1.0, 7.5);
    let material = materials.add(color);
    let model2d = MaterialMesh2dBundle {
        mesh: meshes.add(shape).into(),
        transform,
        material,
        ..default()
    };

    let settings = PlaybackSettings {
        mode: bevy::audio::PlaybackMode::Despawn,
        speed: 1.5,
        volume: Volume::new(0.3),
        ..Default::default()
    };
    let pew_sound = AudioBundle {
        source: assets.laser_sound.clone(),
        settings,
    };
    // audio

    cmds.spawn(pew_sound);
    let missle = (
        // moving_obj,
        plasma,
        ColliderDensity(Plasma::DENSITY),
        RigidBody::Dynamic,
        collider,
        velocity,
        model2d,
        // HomeMadeCollider::new(0.1),
        Health {
            life: 1,
            death_cry: DeathCry::Pop,
        },
        CollisionDamage(Plasma::DAMAGE),
    );
    cmds.spawn(missle);
}
fn cooldown_guns<T: Gun + Component>(mut q: Query<&mut T>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for mut gun in q.iter_mut() {
        gun.cooldown(dt)
    }
}
