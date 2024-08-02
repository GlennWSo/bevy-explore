use std::borrow::BorrowMut;

use crate::{collide_dmg::CollisionDamage, health::Health, schedule::InGameSet, ship::Player};

use super::{handle_gun_fire, FireCtrl, GunFireEvent, MyAssets, SpawnMissle};

use avian2d::prelude::*;
use bevy::{
    audio,
    prelude::{Entity, *},
    sprite::{Material2d, MaterialMesh2dBundle},
};

pub struct NinjaPlugin;

impl Plugin for NinjaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_hook_fire);
        app.add_event::<GunFireEvent<NinjaGun>>();
        app.add_event::<ReleaseHookEvent>();
        app.add_systems(Update, stick_on_collide.in_set(InGameSet::Spawn));
        app.add_systems(Update, ui_release_hook.in_set(InGameSet::UI));
        app.add_systems(Update, handle_hook_release.in_set(InGameSet::Despawn));
        app.add_systems(
            Update,
            (glue_break, remove_long_hook).in_set(InGameSet::EntityUpdate),
        );
    }
}

#[derive(Component)]
pub struct NinjaHook;

impl NinjaHook {
    const SPEED: f32 = 120.0;
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
    hook: Option<Entity>,
}

#[derive(Component)]
struct Glue {
    on: Entity,
}

#[derive(Event)]
struct ReleaseHookEvent {
    gun: Entity,
}

fn ui_release_hook(
    mut writer: EventWriter<ReleaseHookEvent>,
    btn_input: Res<ButtonInput<KeyCode>>,
    q: Query<(Entity), (With<Player>, With<NinjaGun>)>,
) {
    if !btn_input.pressed(KeyCode::Tab) {
        return;
    }

    let Ok(gun) = q.get_single() else {
        return;
    };
    writer.send(ReleaseHookEvent { gun });
}

fn handle_hook_release(
    mut reader: EventReader<ReleaseHookEvent>,
    mut cmds: Commands,
    mut q: Query<&mut NinjaGun>,
) {
    for ReleaseHookEvent { gun } in reader.read() {
        // gu
        // cmds.entity(*gun).borrow_mut();
        let mut ninja_gun = q.get_mut(*gun).unwrap();
        (*ninja_gun).state = NinjaState::Ready;
        let hook = (*ninja_gun).hook.take();
        if let Some(hook) = hook {
            cmds.entity(hook).despawn_recursive();
        }
    }
}

fn stick_on_collide(
    mut cmds: Commands,
    // mut collision_event_reader: EventReader<CollisionStarted>,
    q: Query<(Entity, &CollidingEntities, &Transform), (With<NinjaHook>, Without<Glue>)>,
    player_q: Query<(Entity, &Transform), With<Player>>,
) {
    let Ok((hook_id, collisions, transform)) = q.get_single() else {
        return;
    };
    let Some(&other_entity) = collisions.iter().next() else {
        return;
    };
    let glue_joint = FixedJoint::new(hook_id, other_entity);
    let glue_joint = cmds.spawn(glue_joint).id();
    cmds.entity(hook_id)
        .insert(Glue { on: other_entity })
        .push_children(&[glue_joint]);

    let Ok(player) = player_q.get_single() else {
        return;
    };
    let distance = player.1.translation.distance(transform.translation) + 20.0;
    let joint = DistanceJoint::new(player.0, hook_id)
        .with_limits(0.0, distance)
        .with_compliance(1e-2);
    cmds.spawn(joint);
}

fn remove_long_hook(
    hook_q: Query<(&FromGun, &Transform), (With<NinjaHook>, Without<Glue>)>,
    gun_q: Query<&Transform, With<NinjaGun>>,
    mut writer: EventWriter<ReleaseHookEvent>,
) {
    let max_dist = 130.0;
    for (gun_id, hook_transform) in hook_q.iter() {
        let Ok(gun_transform) = gun_q.get(**gun_id) else {
            continue;
        };
        let distance = hook_transform
            .translation
            .distance(gun_transform.translation);
        if distance > max_dist {
            println!("hook out of bounds");
            writer.send(ReleaseHookEvent { gun: **gun_id });
        }
    }
}

// maybe it is better to observe on Joints being removed?
fn glue_break(
    q: Query<(&Glue, &FromGun)>,
    q_glued2: Query<()>,
    mut writer: EventWriter<ReleaseHookEvent>,
) {
    for (glue, gun) in q.iter() {
        if let Ok(_other) = q_glued2.get(glue.on) {
            continue;
        }
        writer.send(ReleaseHookEvent { gun: **gun });
    }
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

#[derive(Component, Deref, DerefMut)]
struct FromGun(Entity);

#[derive(Bundle)]
pub struct HookBundle<M: Material2d> {
    pub gun: FromGun,
    pub ninjahook: NinjaHook,
    pub model: MaterialMesh2dBundle<M>,
    pub collider: Collider,
    pub rigidbody: RigidBody,
    pub density: ColliderDensity,
    pub velocity: LinearVelocity,
}
impl NinjaGun {
    fn spawn_missle(
        &mut self,
        parrent: Entity,
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

        let missle = HookBundle {
            model,
            collider: Collider::capsule(radius, length),
            rigidbody: RigidBody::Dynamic,
            density: ColliderDensity(NinjaHook::DENSITY),
            velocity,
            ninjahook: NinjaHook,
            gun: FromGun(parrent),
        };
        let missle_id = cmds.spawn(missle).id();
        self.hook = Some(missle_id);
        self.pew(cmds, assets);
    }

    fn pew(&self, cmds: &mut Commands, assets: &Res<MyAssets>) {
        let settings = PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            speed: 1.5,
            volume: audio::Volume::new(0.3),
            ..Default::default()
        };
        let pew_sound = AudioBundle {
            source: assets.doing.clone(),
            settings,
        };
        // audio

        cmds.spawn(pew_sound);
    }
}
fn handle_hook_fire(
    mut reader: EventReader<GunFireEvent<NinjaGun>>,
    mut cmds: Commands,
    mut q: Query<(Entity, &mut NinjaGun, &LinearVelocity, &Transform)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<MyAssets>,
) {
    reader.read().for_each(|event| {
        let Ok(res) = q.get_mut(event.entity) else {
            return;
        };
        let (gun_id, mut gun, ship_velocity, _) = res;

        let Some(_) = gun.fire() else {
            return;
        };
        let mut origin = event.origin;
        origin.translation.z = 10.0;
        gun.spawn_missle(
            gun_id,
            &mut cmds,
            ship_velocity,
            origin,
            &mut materials,
            &mut meshes,
            &assets,
        );
        // cmds.entity(event.entity).push_children(&[missle_id]);
    });
}
