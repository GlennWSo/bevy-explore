use crate::{collide_dmg::CollisionDamage, health::Health};

use super::{FireCtrl, MissleBundle, MyAssets, SpawnMissle};

use avian2d::prelude::*;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

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
                life: 1,
                ..default()
            },
            damage: CollisionDamage(0),
            velocity,
        };
        cmds.spawn(missle);
        // self.pew(cmds, assets);
    }
}
