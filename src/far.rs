use bevy::prelude::*;

/// used for marking entity to not be faraway removed
#[derive(Component)]
pub struct Keep;

const MAX_DISTANCE: f32 = 100.0;

fn remove_far(mut cmds: Commands, q: Query<(Entity, &Transform), Without<Keep>>) {
    for (ent, trans) in q.iter() {
        let distance = trans.translation.distance(Vec3::ZERO);
        if distance > MAX_DISTANCE {
            cmds.entity(ent).despawn_recursive();
        }
    }
}

pub struct FarPlugin;

impl Plugin for FarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, remove_far);
    }
}
