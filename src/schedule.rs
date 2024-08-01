use bevy::prelude::*;

use crate::state::GameState;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InGameSet {
    UI,
    EntityUpdate,
    CollisionDetection,
    Spawn,
    Despawn,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InitStages {
    LoadAssets,
    Spawn,
}

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Startup, (InitStages::LoadAssets, InitStages::Spawn).chain());
        app.configure_sets(
            Update,
            (
                InGameSet::Despawn,
                // flush here
                InGameSet::UI,
                InGameSet::EntityUpdate,
                InGameSet::Spawn,
                InGameSet::CollisionDetection,
            )
                .chain()
                // .run_if(derp),
                .run_if(in_state(GameState::Play)),
        )
        .add_systems(
            Update,
            apply_deferred
                .after(InGameSet::Despawn)
                .before(InGameSet::UI),
        );
    }
}
