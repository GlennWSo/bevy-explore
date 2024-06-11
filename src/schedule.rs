use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InGameSet {
    UI,
    EntityUpdate,
    CollisionDetection,
    Despawn,
}

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        let ordering = (
            InGameSet::Despawn,
            // flush here
            InGameSet::UI,
            InGameSet::EntityUpdate,
            InGameSet::CollisionDetection,
        )
            .chain();
        app.configure_sets(Update, ordering).add_systems(
            Update,
            apply_deferred
                .after(InGameSet::Despawn)
                .before(InGameSet::UI),
        );
    }
}
