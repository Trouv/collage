use std::marker::PhantomData;

use bevy::asset::InvalidGenerationError;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

/// `Effect` that adds an asset to an `Assets` resource and can produce more effects with the
/// `Handle`.
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone, Hash, Reflect)]
pub struct AssetsAddAnd<A, E, F>
where
    A: Asset,
    E: Effect,
    F: FnOnce(Handle<A>) -> E,
{
    pub asset: A,
    pub f: F,
}

/// Construct a new [`AssetsAddAnd`] [`Effect`].
pub fn assets_add_and<A, E, F>(asset: A, f: F) -> AssetsAddAnd<A, E, F>
where
    A: Asset,
    E: Effect,
    F: FnOnce(Handle<A>) -> E,
{
    AssetsAddAnd { asset, f }
}

impl<A, E, F> Effect for AssetsAddAnd<A, E, F>
where
    A: Asset,
    E: Effect,
    F: FnOnce(Handle<A>) -> E,
{
    type MutParam = (ResMut<'static, Assets<A>>, E::MutParam);

    fn affect(self, param: &mut <Self::MutParam as bevy::ecs::system::SystemParam>::Item<'_, '_>) {
        let handle = param.0.add(self.asset);
        (self.f)(handle).affect(&mut param.1);
    }
}

/// `Effect` that sets an existing asset to a new value.
#[derive(Default, Debug, PartialEq, Eq, Clone, Hash, Reflect)]
pub struct AssetsInsert<A>
where
    A: Asset,
{
    pub handle: Handle<A>,
    pub asset: A,
}

/// Construct a new [`AssetsInsert`] [`Effect`].
pub fn assets_insert<A: Asset>(handle: Handle<A>, asset: A) -> AssetsInsert<A> {
    AssetsInsert { handle, asset }
}

impl<A> Effect for AssetsInsert<A>
where
    A: Asset,
{
    type MutParam = (
        ResMut<'static, Assets<A>>,
        <Result<(), InvalidGenerationError> as Effect>::MutParam,
    );

    fn affect(self, param: &mut <Self::MutParam as bevy::ecs::system::SystemParam>::Item<'_, '_>) {
        match param.0.insert(&self.handle, self.asset) {
            Ok(()) => (),
            e => e.affect(&mut param.1),
        }
    }
}
