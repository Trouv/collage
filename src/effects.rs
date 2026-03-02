use bevy::asset::InvalidGenerationError;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

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
