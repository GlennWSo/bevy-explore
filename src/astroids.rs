use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;

use rand::Rng;

use crate::assets::MyAssets;
// use crate::collide::Collider;
use crate::collide::CollisionDamage;
use crate::collide::HomeMadeCollider;
use crate::health::Health;
use crate::movement::MovingObj;
use crate::schedule::InGameSet;
use crate::zones::Extra;
use crate::zones::IntoMovingBundle;
use crate::zones::Stage;

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

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
pub struct Astroid {
    pub bulk: u8,
}

impl Astroid {
    const ROTATION_SPEED: f32 = 1.0;
    const RADIUS_MOD: f32 = 2.5;
    /// hit point scaling with size
    const LIFE_MOD: i32 = 10;
    /// hit point scaling with size
    const DAMAGE_MOD: i32 = 5;

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
        CollisionDamage(self.bulk as i32 * Self::DAMAGE_MOD)
    }

    fn health(&self) -> Health {
        Health {
            life: self.bulk as i32 * Self::LIFE_MOD,
            ..Default::default()
        }
    }

    fn collider(&self) -> (Collider, HomeMadeCollider) {
        (Collider::circle(1.0), HomeMadeCollider::new(self.radius()))
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

fn rotate_astriods(mut q: Query<&mut Transform, With<Astroid>>, time: Res<Time>) {
    let rot = Astroid::ROTATION_SPEED * time.delta_seconds();
    for mut trans in q.iter_mut() {
        trans.rotate_local_z(rot);
    }
}

// #[derive(Component, Clone, Copy)]
// struct Shard;

fn split_dead(
    mut cmds: Commands,
    q: Query<(&Health, &Transform, &LinearVelocity, &Astroid)>,
    assets: Res<MyAssets>,
) {
    for (health, &transform, &velocity, astriod) in q.iter() {
        if **health > 0 {
            continue;
        }
        let velicities = explode_veclocity(*velocity, astriod.bulk as usize - 1);
        let particles = velicities.into_iter().map(|v| {
            let origin = transform.translation.truncate();
            let c = 5.;
            let offset = v.normalize() * (astriod.bulk as f32 / 1. + c);
            let spawn_coord = origin + offset;
            (spawn_coord, v)
        });
        let astroid = Astroid { bulk: 1 };
        astroid.spawn(&assets, particles, &mut cmds);
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
            let angle_mod = rng.gen_range(0.8..1.25);
            let rot = Quat::from_rotation_z(section_angle.to_radians() * angle_mod);
            v = rot.mul_vec3(v);
            v.truncate() * speed_mod + origin_velocity
        })
        .collect()
}

impl Extra for Astroid {
    type Extras = (
        Health,
        (Collider, HomeMadeCollider),
        CollisionDamage,
        Name,
        RigidBody,
    );

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
        let mesh = Mesh2dHandle(assets.ball.clone());
        let model2d = MaterialMesh2dBundle {
            mesh,
            transform,
            material: assets.asteroid_material.clone(),
            visibility: Visibility::Visible,
            ..Default::default()
        };
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
    mover: MovingObj,
    astriod: Astroid,
    damage: CollisionDamage,
    health: Health,
    name: Name,
}
