use std::marker::PhantomData;

use bevy::asset::InvalidGenerationError;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

/// [`Effect`] that inserts a component recursively on relationship targets.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct EntityCommandInsertRecursive<RT, B>
where
    RT: RelationshipTarget,
    B: Bundle + Clone,
{
    pub entity: Entity,
    pub bundle: B,
    _phantom: PhantomData<RT>,
}

impl<RT, B> EntityCommandInsertRecursive<RT, B>
where
    B: Bundle + Clone,
    RT: RelationshipTarget,
{
    /// Construct a new [`EntityCommandInsertRecursive`].
    fn new(entity: Entity, bundle: B) -> Self {
        Self {
            entity,
            bundle,
            _phantom: PhantomData,
        }
    }
}

/// Construct a new [`EnttiyCommandInsertRecursive`] [`Effect`].
pub fn entity_command_insert_recursive<RT, B>(
    entity: Entity,
    bundle: B,
) -> EntityCommandInsertRecursive<RT, B>
where
    B: Bundle + Clone,
    RT: RelationshipTarget,
{
    EntityCommandInsertRecursive::new(entity, bundle)
}

impl<RT, B> Effect for EntityCommandInsertRecursive<RT, B>
where
    RT: RelationshipTarget,
    B: Bundle + Clone,
{
    type MutParam = Commands<'static, 'static>;

    fn affect(self, param: &mut <Self::MutParam as bevy::ecs::system::SystemParam>::Item<'_, '_>) {
        param
            .entity(self.entity)
            .insert_recursive::<RT>(self.bundle);
    }
}

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
