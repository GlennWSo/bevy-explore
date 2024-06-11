use std::ops::Deref;

use bevy::{prelude::*, utils::HashMap};

use crate::{
    astroids::Astroid,
    health::Health,
    schedule::InGameSet,
    ship::{Missle, SpaceShip},
};

pub struct CollidePlugin;

impl Plugin for CollidePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionEvent>()
            .add_systems(
                Update,
                detect_collisions.in_set(InGameSet::CollisionDetection),
            )
            .add_systems(
                Update,
                (
                    (
                        handle_collisions::<SpaceShip>,
                        handle_collisions::<Astroid>,
                        handle_collisions::<Missle>,
                    ),
                    apply_collision_dmg,
                )
                    .chain()
                    .in_set(InGameSet::Despawn),
            );
    }
}

#[derive(Component)]
pub struct CollisionDamage(pub i32);

impl Deref for CollisionDamage {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Event)]
struct CollisionEvent {
    entity: Entity,
    collided_with: Entity,
}

impl CollisionEvent {
    pub fn new(entity: Entity, collided_with: Entity) -> Self {
        Self {
            entity,
            collided_with,
        }
    }
}

#[derive(Component, Default)]
pub struct Collider {
    pub radius: f32,
    pub colliding_entities: Vec<Entity>,
}

impl Collider {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            ..Default::default()
        }
    }
}

fn detect_collisions(mut q: Query<(Entity, &GlobalTransform, &mut Collider)>) {
    let mut collisions_map: HashMap<Entity, Vec<Entity>> = HashMap::new();

    // first phase dectect collisions
    for (ent_a, trans_a, collider_a) in q.iter() {
        for (ent_b, trans_b, collider_b) in q.iter() {
            if ent_a == ent_b {
                continue;
            }
            let dist = (trans_a.translation() - trans_b.translation()).length();
            let limit = collider_a.radius + collider_b.radius;
            if dist < limit {
                collisions_map
                    .entry(ent_a)
                    .or_insert_with(Vec::new)
                    .push(ent_b);
            }
        }
    }

    for (ent, _, mut collider) in q.iter_mut() {
        collider.colliding_entities.clear();
        if let Some(collisions) = collisions_map.get(&ent) {
            collider.colliding_entities.extend(collisions.iter());
        }
    }
}

fn handle_collisions<T: Component>(
    mut writer: EventWriter<CollisionEvent>,
    q: Query<(Entity, &Collider), With<T>>,
) {
    for (ent, collider) in q.iter() {
        for &collide_envent in collider.colliding_entities.iter() {
            if q.get(collide_envent).is_ok() {
                continue;
            }
            // send collision event
            writer.send(CollisionEvent::new(ent, collide_envent));
        }
    }
}

fn apply_collision_dmg(
    mut reader: EventReader<CollisionEvent>,
    mut health_q: Query<&mut Health>,
    dmg_q: Query<&CollisionDamage>,
) {
    for &CollisionEvent {
        entity,
        collided_with,
    } in reader.read()
    {
        let Ok(mut health) = health_q.get_mut(entity) else {
            continue;
        };

        let Ok(dmg) = dmg_q.get(collided_with) else {
            continue;
        };

        **health -= **dmg;
    }
}
