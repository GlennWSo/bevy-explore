mod ninja;
pub use self::ninja::NinjaGun;

use std::marker::PhantomData;

use avian2d::prelude::*;
use bevy::prelude::Component;
use bevy::reflect::Tuple;
use bevy::sprite::Material2d;
use bevy::{audio::Volume, prelude::*, sprite::MaterialMesh2dBundle};

use crate::collide_dmg::CollisionDamage;
use crate::ship::SpaceShip;
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
        .add_systems(Update, handle_gun_fire::<PlasmaGun>)
        .add_systems(Update, handle_gun_fire::<NinjaGun>)
        .add_systems(Update, ship_weapon_ctrl.in_set(InGameSet::UI))
        .add_systems(Update, despawn_far::<Plasma, 10_000>);
        app.add_event::<GunFireEvent<PlasmaGun>>();
        app.add_event::<GunFireEvent<NinjaGun>>();
    }
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

impl PlasmaGun {
    fn pew(&self, cmds: &mut Commands, assets: &Res<MyAssets>) {
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
    }
}

#[derive(Bundle)]
pub struct MissleBundle<M: Material2d> {
    pub model: MaterialMesh2dBundle<M>,
    pub collider: Collider,
    pub rigidbody: RigidBody,
    pub density: ColliderDensity,
    pub health: Health,
    pub damage: CollisionDamage,
    pub velocity: LinearVelocity,
}

pub trait FireCtrl {
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

impl FireCtrl for PlasmaGun {
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

impl SpawnMissle for PlasmaGun {
    fn spawn_missle(
        &self,
        cmds: &mut Commands,
        ship_velocity: &LinearVelocity,
        origin: Transform,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        assets: &Res<MyAssets>,
    ) {
        let radius = 0.5;
        let length = 2.;
        let shape = Capsule2d::new(radius, length);
        let color = Color::srgb(7.5, 1.0, 7.5);
        let material = materials.add(color);
        let model = MaterialMesh2dBundle {
            mesh: meshes.add(shape).into(),
            transform: origin,
            material,
            ..default()
        };
        let velocity: LinearVelocity =
            (-origin.up().truncate() * Plasma::SPEED + **ship_velocity).into();

        let missle = MissleBundle {
            model,
            collider: Collider::capsule(radius, length),
            rigidbody: RigidBody::Dynamic,
            density: ColliderDensity(Plasma::DENSITY),
            health: Health {
                life: 1,
                ..default()
            },
            damage: CollisionDamage(Plasma::DAMAGE),
            velocity,
        };
        cmds.spawn(missle);
        self.pew(cmds, assets);
    }
}

#[derive(Event)]
struct GunFireEvent<G: FireCtrl + SpawnMissle> {
    phantom: PhantomData<G>,
    entity: Entity,
    origin: Transform,
}

fn ship_weapon_ctrl(
    q: Query<(Entity, &Transform), With<Player>>,
    mut plasa_events: EventWriter<GunFireEvent<PlasmaGun>>,
    mut hook_events: EventWriter<GunFireEvent<NinjaGun>>,

    btn_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok((entity, ship_transform)) = q.get_single() else {
        return;
    };
    let translation = ship_transform.translation - ship_transform.up() * FORWARD_OFFSET;
    let mut origin = ship_transform.clone();
    origin.scale = [1., 1., 1.].into();
    origin.translation = translation;

    if btn_input.pressed(KeyCode::Space) {
        plasa_events.send(GunFireEvent {
            entity,
            origin,
            phantom: PhantomData,
        });
    }
    if btn_input.pressed(KeyCode::ControlLeft) {
        hook_events.send(GunFireEvent {
            entity,
            origin,
            phantom: PhantomData,
        });
        println!("Fire hook from: {}", entity);
    }
}

trait SpawnMissle {
    fn spawn_missle(
        &self,
        cmds: &mut Commands,
        velocity: &LinearVelocity,
        origin: Transform,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        assets: &Res<MyAssets>,
    );
}

trait Gun = FireCtrl + SpawnMissle + Component;

fn handle_gun_fire<G: Gun>(
    mut reader: EventReader<GunFireEvent<G>>,
    mut cmds: Commands,
    mut q: Query<(&mut G, &LinearVelocity, &Transform)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<MyAssets>,
) {
    reader.read().for_each(|event| {
        let Ok(res) = q.get_mut(event.entity) else {
            return;
        };
        let (mut gun, ship_velocity, ship_transform) = res;

        let Some(plasma) = gun.fire() else {
            return;
        };
        gun.spawn_missle(
            &mut cmds,
            ship_velocity,
            event.origin,
            &mut materials,
            &mut meshes,
            &assets,
        );
    });
}
fn fire_hook(
    mut hook_gun: Mut<NinjaGun>,
    ship_transform: &Transform,
    ship_velocity: &LinearVelocity,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    assets: &Res<MyAssets>,
    cmds: &mut Commands,
) {
    let Some(plasma) = hook_gun.fire() else {
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
fn cooldown_guns<T: FireCtrl + Component>(mut q: Query<&mut T>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for mut gun in q.iter_mut() {
        gun.cooldown(dt)
    }
}
