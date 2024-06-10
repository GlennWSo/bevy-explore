use bevy::{prelude::*, utils::HashMap};

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

pub struct CollidePlugin;

impl Plugin for CollidePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, detect_collisions);
    }
}
