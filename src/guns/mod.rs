mod ninja;
mod plasma;
pub use self::ninja::NinjaGun;
use self::ninja::NinjaPlugin;
// use self::plasma::Plasma;
pub use self::plasma::{Plasma, PlasmaGun};

use std::marker::PhantomData;

use avian2d::prelude::*;
use bevy::prelude::Component;
use bevy::sprite::Material2d;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::collide_dmg::CollisionDamage;
use crate::health::cry_dead;
use crate::{
    assets::MyAssets, despawn::despawn_far, health::Health, schedule::InGameSet, ship::Player,
};

const FORWARD_OFFSET: f32 = 8.5;

pub struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            cooldown_guns::<PlasmaGun>.in_set(InGameSet::EntityUpdate),
        )
        .add_systems(Update, handle_gun_fire::<PlasmaGun>)
        .add_systems(Update, cry_dead::<Plasma>.in_set(InGameSet::Spawn))
        .add_systems(Update, despawn_far::<Plasma, 10_000>);
        app.add_event::<GunFireEvent<PlasmaGun>>();
        app.add_plugins(NinjaPlugin);
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

#[derive(Event)]
pub struct GunFireEvent<G: FireCtrl> {
    pub phantom: PhantomData<G>,
    pub entity: Entity,
    pub transform: Transform,
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
    ) -> Entity;
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
        let (mut gun, ship_velocity, _) = res;

        let Some(_) = gun.fire() else {
            return;
        };
        gun.spawn_missle(
            &mut cmds,
            ship_velocity,
            event.transform,
            &mut materials,
            &mut meshes,
            &assets,
        );
    });
}

fn cooldown_guns<T: FireCtrl + Component>(mut q: Query<&mut T>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for mut gun in q.iter_mut() {
        gun.cooldown(dt)
    }
}
