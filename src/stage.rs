use avian2d::prelude::*;
use bevy::prelude::*;

use crate::assets::MyAssets;

pub trait Extra {
    type Extras: Bundle + Sized;
    fn extra(&self) -> Self::Extras;
}

pub trait Stage {
    fn stage(self, assets: &Res<MyAssets>, transform: Transform) -> impl Bundle;
}
pub trait IntoMovingBundle {
    type Extras: Bundle + Sized;
    fn bundle(self, assets: &Res<MyAssets>, transform: Transform, velocity: Vec2) -> impl Bundle;
}

impl<C, T> IntoMovingBundle for T
where
    C: Bundle + Sized,
    T: Extra<Extras = C> + Stage + Copy + Bundle,
{
    type Extras = C;

    fn bundle(self, assets: &Res<MyAssets>, transform: Transform, velocity: Vec2) -> impl Bundle {
        (
            self.extra(),
            self.stage(assets, transform),
            LinearVelocity(velocity),
            self,
        )
    }
}
