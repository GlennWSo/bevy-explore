use crate::{
    collide_dmg::CollisionDamage,
    health::{DeathCry, Health},
    layers::GameLayer,
};

use super::{FireCtrl, MissleBundle, MyAssets, SpawnMissle};

use avian2d::prelude::*;
use bevy::{audio::Volume, prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Component)]
pub struct Plasma;

impl Plasma {
    const SPEED: f32 = 80.0;
    const DAMAGE: i32 = 10;
    const DENSITY: f32 = 5.0;
}

impl DeathCry for Plasma {
    fn cry(&self, assets: &MyAssets) -> AudioBundle {
        AudioBundle {
            source: assets.pop.clone(),
            settings: PlaybackSettings::DESPAWN,
        }
    }
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
    ) -> Entity {
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
        let memberships = dbg!(LayerMask(GameLayer::Plasma.to_bits()));
        let filters = dbg!(LayerMask::ALL & !memberships);
        let layer = CollisionLayers {
            memberships,
            filters,
        };

        let id = cmds.spawn((Plasma, missle, layer)).id();
        self.pew(cmds, assets);
        id
    }
}
