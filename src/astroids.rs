use avian2d::prelude::*;
use bevy::prelude::*;

use rand::Rng;
use rand_distr::Distribution;
use rand_distr::Standard;

use crate::assets::MyAssets;
use crate::collide_dmg::CollisionDamage;
// use crate::collide::Collider;
// use crate::collide::CollisionDamage;
// use crate::collide::HomeMadeCollider;
use crate::health::Health;
use crate::schedule::InGameSet;
use crate::stage::Extra;
use crate::stage::IntoMovingBundle;
use crate::stage::Stage;

pub struct AstriodPlug;

impl Plugin for AstriodPlug {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, split_dead.in_set(InGameSet::Spawn));
        // .add_systems(Update, despawn_astroid.in_set(InGameSet::Despawn))
        // .add_systems(
        //     Update,
        //     despawn::despawn_far::<Astroid, 1000>.in_set(InGameSet::Despawn),
        // )
        // .add_systems(Update, rotate_astriods.in_set(InGameSet::EntityUpdate));
    }
}

#[derive(Component, Default, Debug, Copy, Clone, Reflect, PartialEq, Eq, Hash)]
pub enum Rock {
    #[default]
    Stone,
    Ice,
    Metal,
}

impl Distribution<Rock> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Rock {
        // let stone_prob = 800;
        const ICE_PROB: u32 = 300;
        const METAL_PROB: u32 = 100;
        const METAL_HB: u32 = METAL_PROB + ICE_PROB;

        match rng.gen_range(0..1000) {
            0..ICE_PROB => Rock::Ice,
            ICE_PROB..METAL_HB => Rock::Metal,
            _ => Rock::Stone,
        }
    }
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
pub struct Astroid {
    pub bulk: u8,
    pub kind: Rock,
}

impl Astroid {
    // const ROTATION_SPEED: f32 = 1.0;
    const RADIUS_MOD: f32 = 2.5;
    /// hit point scaling with size
    const LIFE_MOD: i32 = 10;
    /// hit point scaling with size

    fn spawn(
        &self,
        assets: &Res<MyAssets>,
        particles: impl Iterator<Item = (Vec2, Vec2)>,
        cmds: &mut Commands,
    ) {
        let batch: Box<[_]> = particles
            .map(|particle| {
                let transform = Transform::from_translation(particle.0.extend(0.0));
                self.bundle(assets, transform, particle.1)
            })
            .collect();
        cmds.spawn_batch(batch);
    }

    fn damage(&self) -> CollisionDamage {
        // CollisionDamage(self.bulk as i32 * Self::DAMAGE_MOD)
        CollisionDamage(1)
    }

    fn health(&self) -> Health {
        Health {
            life: self.bulk as i32 * Self::LIFE_MOD,
            ..Default::default()
        }
    }

    fn collider(&self) -> Collider {
        Collider::circle(1.0)
    }

    fn radius(&self) -> f32 {
        (self.bulk as f32).sqrt() * Self::RADIUS_MOD
    }

    const SPEED_MOD: f32 = 5.0;
    pub fn random_velocity() -> Vec2 {
        let mut rng = rand::thread_rng();

        let v_unit = random_unit_vec(&mut rng);
        let factor: f32 = rng.gen_range(0.0..Self::SPEED_MOD);
        v_unit * factor
        // Velocity::default()
    }

    fn scale(&self) -> Vec3 {
        Vec3::splat(self.radius())
    }
}

fn random_unit_vec(rng: &mut impl Rng) -> Vec2 {
    let x = rng.gen_range(-1.0..1.0);
    let y = rng.gen_range(-1.0..1.0);
    Vec2::new(x, y).normalize_or_zero()
}

fn split_dead(
    mut cmds: Commands,
    q: Query<(&Health, &Transform, &LinearVelocity, &Astroid)>,
    assets: Res<MyAssets>,
) {
    for (health, &transform, &velocity, astriod) in q.iter() {
        if **health > 0 {
            continue;
        }
        if astriod.bulk <= 1 {
            continue;
        }
        let velicities = explode_veclocity(*velocity, 2);
        let shard = Astroid {
            bulk: astriod.bulk / 2,
            kind: astriod.kind,
        };
        let particles = velicities.into_iter().map(|v| {
            let origin = transform.translation.truncate();
            let c = 0.5;

            let offset = (v - *velocity).normalize() * (shard.radius() * 1.1 + c);
            let spawn_coord = origin + offset;
            (spawn_coord, v)
        });
        shard.spawn(&assets, particles, &mut cmds);
        let sound = AudioBundle {
            source: assets.crack.clone(),
            settings: PlaybackSettings::DESPAWN,
        };
        cmds.spawn(sound);
    }
}

/// create vectors moving away from vector
fn explode_veclocity(origin_velocity: Vec2, n: usize) -> Vec<Vec2> {
    let mut rng = rand::thread_rng();
    let base_speed: f32 = rng.gen_range(2.5..10.);

    let mut v = (random_unit_vec(&mut rng) * base_speed).extend(0.0);
    let section_angle = 360.0 / n as f32;
    // let rot = Quat::from_rotation_y(angle.to_radians());

    (0..n)
        .map(|_| {
            // let speed_mod = 1.;
            // let angle_mod = 1.;
            let speed_mod = rng.gen_range(0.8..1.25);
            // let angle_mod = rng.gen_range(0.8..1.25);
            let rot = Quat::from_rotation_z(section_angle.to_radians());
            v = rot.mul_vec3(v);
            v.truncate() * speed_mod + origin_velocity
        })
        .collect()
}

impl Extra for Astroid {
    type Extras = (Health, Collider, CollisionDamage, Name, RigidBody);

    fn extra(&self) -> Self::Extras {
        (
            self.health(),
            self.collider(),
            self.damage(),
            Name::new("Astroid"),
            RigidBody::Dynamic,
        )
    }
}

impl Stage for Astroid {
    fn stage(
        self,
        assets: &Res<MyAssets>, //
        transform: Transform,
    ) -> impl Bundle {
        let transform = transform.with_scale(self.scale());
        // let mesh = Mesh2dHandle(assets.ball.clone());

        // let texture = assets.astriod.clone();
        println!("{:?}", self.kind);
        let texture = match self.kind {
            Rock::Stone => assets.astriod.clone(),
            Rock::Ice => assets.astriod2.clone(),
            Rock::Metal => assets.astriod_metal.clone(),
        };
        let model2d = SpriteBundle {
            transform,
            texture,
            sprite: Sprite {
                custom_size: Some(Vec2 { x: 2., y: 2. }),
                ..default()
            },
            ..default()
        };
        // let model2d = MaterialMesh2dBundle {
        //     mesh,
        //     transform,
        //     material: assets.asteroid_material.clone(),
        //     visibility: Visibility::Visible,
        //     ..Default::default()
        // };
        model2d
        // SceneBundle {
        //     transform,
        //     scene: assets.astriod.clone(),
        //     visibility: Visibility::Visible,
        //     ..Default::default()
        // }
    }
}

impl From<AstriodBundle> for Astroid {
    fn from(bundle: AstriodBundle) -> Self {
        bundle.astriod
    }
}

#[derive(Bundle)]
struct AstriodBundle {
    astriod: Astroid,
    damage: CollisionDamage,
    health: Health,
    name: Name,
}
