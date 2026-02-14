use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::paint_skies::paint_meshes::LayerIndex;

/// Plugin that tracks the history of a component at previous paint layers.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct PaintLayerHistoryPlugin<C>(PhantomData<C>);

impl<C> Plugin for PaintLayerHistoryPlugin<C>
where
    C: Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {}
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Component)]
pub struct PaintableHistory<C> {
    history: Vec<C>,
    initial_layer: LayerIndex,
}
