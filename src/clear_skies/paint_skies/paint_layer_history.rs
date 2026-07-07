use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::reflect::{FromReflect, GetTypeRegistration, Typed};
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::paint_skies::paint_meshes::{LayerIndex, ReadyToPaint};

/// System set for systems that modify paint layer history.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Hash, SystemSet)]
pub struct RecordPaintLayerHistorySet;

/// Core plugin within [`PaintLayerHistoryPlugin`] that doesn't deal with the [`HistoryUnit`] to
/// avoid infinite recursion.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
struct PaintLayerHistoryPluginNoUnit<C>(PhantomData<C>);

impl<C> Plugin for PaintLayerHistoryPluginNoUnit<C>
where
    C: Component + Typed + GetTypeRegistration + FromReflect + Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.register_type::<PaintableHistory<C>>().add_systems(
            Update,
            (
                truncate_history::<C>
                    .pipe(affect)
                    .run_if(on_message::<TruncatePaintLayers>),
                record_history::<C>
                    .pipe(affect)
                    .run_if(on_message::<ReadyToPaint>),
            )
                .chain()
                .in_set(RecordPaintLayerHistorySet),
        );
    }
}

/// Plugin that tracks the history of a component at previous paint layers.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct PaintLayerHistoryPlugin<C>(PhantomData<C>);

impl<C> Plugin for PaintLayerHistoryPlugin<C>
where
    C: Component + Typed + GetTypeRegistration + FromReflect + Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PaintLayerHistoryPluginNoUnit<HistoryUnit>>() {
            app.add_systems(
                OnEnter(ClearSkiesState::Setup),
                (|| command_spawn(HistoryUnit)).pipe(affect),
            )
            .add_plugins(PaintLayerHistoryPluginNoUnit::<HistoryUnit>::default());
        }

        app.add_plugins(PaintLayerHistoryPluginNoUnit(self.0));
    }
}

/// `Component` that stores the history of another component by layer index.
#[derive(Clone, PartialEq, Eq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct PaintableHistory<C> {
    history: Vec<C>,
    initial_layer: LayerIndex,
}

impl<C> PaintableHistory<C> {
    /// Construct a new [`PaintableHistory`] with the given initial layer.
    pub fn new_with_initial_layer(initial_layer: LayerIndex) -> Self {
        PaintableHistory {
            initial_layer,
            history: vec![],
        }
    }

    /// Get the historical value of the component at this layer index.
    pub fn get(&self, LayerIndex(absolute_index): LayerIndex) -> Option<&C> {
        let relative_index = absolute_index.checked_sub(self.initial_layer.0)?;
        let relative_index_usize: usize = relative_index.try_into().ok()?;

        self.history.get(relative_index_usize)
    }

    /// Return the layer index of the last layer, if the history is non-empty.
    pub fn last_layer_index(&self) -> Option<LayerIndex> {
        let len = self.history.len();

        (len > 0).then(|| LayerIndex(*self.initial_layer + len as u32))
    }

    /// Similar to `vec.iter().enumerate()`, returns an iterator that enumerates the history with `LayerIndex`es.
    #[expect(dead_code)]
    pub fn iter_enumerate_layers(&self) -> impl Iterator<Item = (LayerIndex, &C)> {
        self.history
            .iter()
            .enumerate()
            .map(|(i, c)| (LayerIndex(self.initial_layer.0 + i as u32), c))
    }

    /// Returns this [`PaintableHistory`] with only the elements before layer n.
    pub fn truncate(self, LayerIndex(n): LayerIndex) -> Self {
        let PaintableHistory {
            initial_layer,
            mut history,
        } = self;

        let history_index = n.saturating_sub(initial_layer.0);

        history.truncate(history_index as usize);

        PaintableHistory {
            initial_layer,
            history,
        }
    }
}

/// System that records the history of a component if it has a corresponding `PaintableHistory`
/// component.
fn record_history<C>()
-> QueryMap<(&'static PaintableHistory<C>, &'static C), ComponentSet<PaintableHistory<C>>>
where
    C: Component + Clone,
{
    query_map(|(history, c): (&PaintableHistory<C>, &C)| {
        let mut history = history.clone();
        history.history.push(c.clone());
        component_set(history)
    })
}

/// Send this message when you want to remove paint layers.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Message)]
pub struct TruncatePaintLayers {
    /// The layer index to truncate at.
    pub layer: LayerIndex,
}

fn truncate_history<C>() -> MessagesReadAnd<
    TruncatePaintLayers,
    QueryMap<&'static PaintableHistory<C>, ComponentSet<PaintableHistory<C>>>,
>
where
    C: Component + Clone,
{
    messages_read_and(|&TruncatePaintLayers { layer }| {
        query_map(move |paintable_history: &PaintableHistory<C>| {
            component_set(paintable_history.clone().truncate(layer))
        })
    })
}

/// A unit type whose (trivial) history is tracked along with any others. Useful for understanding
/// the universal history state, like with [`last_layer_index`].
#[derive(Clone, PartialEq, Eq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
#[require(Name = "HistoryUnit", PaintableHistory<HistoryUnit>)]
pub struct HistoryUnit;

/// System that returns the last layer index in the history. Pipe this into a system
/// `.after(RecordPaintLayerHistorySet)` to respond to [`ReadyToPaint`] events if you need to know
/// the newest layer index.
///
/// Previously, this information was stored as a resource, but I found it concerning dealing with
/// the same "indexing" state in two different places that could potentially get out of sync if
/// you're not really careful about scheduling..
pub fn last_layer_index(
    paintable_history: Single<&PaintableHistory<HistoryUnit>>,
) -> Option<LayerIndex> {
    paintable_history.last_layer_index()
}
