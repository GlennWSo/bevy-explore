use crate::{collide_dmg::CollisionDamage, health::Health};

use super::{handle_gun_fire, FireCtrl, GunFireEvent, MyAssets, SpawnMissle};

use avian2d::prelude::*;
use bevy::{
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle},
};

pub struct NinjaPlugin;

impl Plugin for NinjaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_gun_fire::<NinjaGun>);
        app.add_event::<GunFireEvent<NinjaGun>>();
        app.add_systems(Update, stick_on_collide);
    }
}

#[derive(Component)]
pub struct NinjaHook;

impl NinjaHook {
    const SPEED: f32 = 80.0;
    const DENSITY: f32 = 5.0;
}

#[derive(Default)]
enum NinjaState {
    #[default]
    Ready,
    Throwing,
    Hooked,
    Cooldown(f32),
}

#[derive(Component, Default)]
pub struct NinjaGun {
    state: NinjaState,
}

#[derive(Component)]
struct Glue;

fn stick_on_collide(
    mut cmds: Commands,
    // mut collision_event_reader: EventReader<CollisionStarted>,
    q: Query<(Entity, &CollidingEntities), (With<NinjaHook>, Without<Glue>)>,
) {
    let Ok((entity, collisions)) = q.get_single() else {
        return;
    };
    let Some(&other_entity) = collisions.iter().next() else {
        return;
    };
    cmds.entity(entity).insert(Glue);
    let constraint = FixedJoint::new(entity, other_entity);
    cmds.spawn(constraint);
}

impl FireCtrl for NinjaGun {
    type Missle = NinjaHook;

    fn fire(&mut self) -> Option<Self::Missle> {
        let (new_state, res) = match self.state {
            NinjaState::Ready => (NinjaState::Throwing, Some(NinjaHook)),
            NinjaState::Throwing => (NinjaState::Throwing, None),
            NinjaState::Hooked => (NinjaState::Cooldown(0.0), None),
            NinjaState::Cooldown(ds) => (NinjaState::Cooldown(ds), None),
        };
        self.state = new_state;
        res
    }

    fn cooldown(&mut self, dt: f32) {
        todo!()
    }
}
#[derive(Bundle)]
pub struct MissleBundle<M: Material2d> {
    pub ninjahook: NinjaHook,
    pub model: MaterialMesh2dBundle<M>,
    pub collider: Collider,
    pub rigidbody: RigidBody,
    pub density: ColliderDensity,
    pub health: Health,
    pub velocity: LinearVelocity,
}
impl SpawnMissle for NinjaGun {
    fn spawn_missle(
        &self,
        cmds: &mut Commands,
        ship_velocity: &LinearVelocity,
        origin: Transform,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        assets: &Res<MyAssets>,
    ) {
        println!("spawning hook");
        let radius = 0.5;
        let length = 2.;
        let shape = Capsule2d::new(radius, length);
        let color = Color::srgb(0., 0., 10.0);
        let material = materials.add(color);
        let model = MaterialMesh2dBundle {
            mesh: meshes.add(shape).into(),
            transform: origin,
            material,
            ..default()
        };
        let velocity: LinearVelocity =
            (-origin.up().truncate() * NinjaHook::SPEED + **ship_velocity).into();

        let missle = MissleBundle {
            model,
            collider: Collider::capsule(radius, length),
            rigidbody: RigidBody::Dynamic,
            density: ColliderDensity(NinjaHook::DENSITY),
            health: Health {
                life: 100,
                ..default()
            },
            velocity,
            ninjahook: NinjaHook,
        };
        cmds.spawn(missle);
        // self.pew(cmds, assets);
    }
}
