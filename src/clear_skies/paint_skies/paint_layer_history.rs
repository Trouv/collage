use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::paint_skies::paint_meshes::{LayerIndex, ReadyToPaint};

/// Plugin that tracks the history of a component at previous paint layers.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct PaintLayerHistoryPlugin<C>(PhantomData<C>);

impl<C> Plugin for PaintLayerHistoryPlugin<C>
where
    C: Component + Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            record_history::<C>
                .pipe(affect)
                .run_if(on_message::<ReadyToPaint>),
        );
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Component)]
pub struct PaintableHistory<C> {
    history: Vec<C>,
    initial_layer: LayerIndex,
}

impl<C> PaintableHistory<C> {
    pub fn new_with_initial_layer(initial_layer: LayerIndex) -> Self {
        PaintableHistory {
            initial_layer,
            history: vec![],
        }
    }

    pub fn get(&self, LayerIndex(absolute_index): LayerIndex) -> Option<&C> {
        let relative_index = absolute_index.checked_sub(self.initial_layer.0)?;
        let relative_index_usize: usize = relative_index.try_into().ok()?;

        self.history.get(relative_index_usize)
    }
}

fn record_history<C>() -> ComponentsSetWithQueryData<(PaintableHistory<C>,), &'static C>
where
    C: Component + Clone,
{
    components_set_with_query_data(|(mut history,): (PaintableHistory<C>,), c: &C| {
        history.history.push(c.clone());
        (history,)
    })
}
