use core::cmp::Ord;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::reflect::{FromReflect, GetTypeRegistration, Typed};
use bevy_pipe_affect::prelude::*;

use crate::clear_skies::ClearSkiesState;
use crate::clear_skies::paint_skies::paint_meshes::LayerIndex;

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
        app.register_type::<PaintableHistory<C>>()
            .add_message::<TruncatePaintLayers>()
            .add_message::<RecordPresent>()
            .add_systems(
                Update,
                (
                    truncate_history::<C>
                        .pipe(affect)
                        .run_if(on_message::<TruncatePaintLayers>),
                    record_present::<C>
                        .pipe(affect)
                        .run_if(on_message::<RecordPresent>),
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
    history: Vec<Option<C>>,
}

impl<C> PaintableHistory<C> {
    /// Get the historical value of the component at this layer index.
    pub fn get(&self, LayerIndex(absolute_index): LayerIndex) -> Option<&C> {
        self.history.get(absolute_index as usize)?.as_ref()
    }

    /// Return the layer index of the last layer, if the history is non-empty.
    pub fn last_layer_index(&self) -> Option<LayerIndex> {
        let len = self.history.len();

        (len > 0).then(|| LayerIndex(len as u32 - 1))
    }

    /// Similar to `vec.iter().enumerate()`, returns an iterator that enumerates the history with `LayerIndex`es.
    #[expect(dead_code)]
    pub fn iter_enumerate_layers(&self) -> impl Iterator<Item = (LayerIndex, Option<&C>)> {
        self.history
            .iter()
            .enumerate()
            .map(|(i, c)| (LayerIndex(i as u32), c.as_ref()))
    }

    /// Returns this [`PaintableHistory`] with only the elements before layer n.
    pub fn truncate(self, LayerIndex(n): LayerIndex) -> Self {
        let PaintableHistory { mut history } = self;

        history.truncate(n as usize);

        PaintableHistory { history }
    }

    /// Returns this [`PaintableHistory`] with the given value/index as its new ending.
    ///
    /// If the layer index is lower than the current last layer index, the new history will be
    /// truncated first.
    ///
    /// If the layer index is much higher than the current history length, the new history will
    /// have `None`s in the interim.
    pub fn with_end(self, n: LayerIndex, value: Option<C>) -> Self {
        let PaintableHistory { mut history } = self.truncate(n);

        history.extend(std::iter::repeat_with(|| None).take(n.0 as usize - history.len()));

        history.push(value);

        PaintableHistory { history }
    }
}
/// Send this message when you want to record a new layer.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Message)]
pub struct RecordPresent {
    /// The layer index to record as the new present.
    pub layer: LayerIndex,
}

/// System that records the present value of a component into the history if it has a corresponding
/// `PaintableHistory` component.
fn record_present<C>() -> MessagesReadAnd<
    RecordPresent,
    QueryMap<(&'static PaintableHistory<C>, Option<&'static C>), ComponentSet<PaintableHistory<C>>>,
>
where
    C: Component + Clone,
{
    messages_read_and(|RecordPresent { layer }| {
        let layer = *layer;
        query_map(move |(history, c): (&PaintableHistory<C>, Option<&C>)| {
            component_set(history.clone().with_end(layer, c.cloned()))
        })
    })
}

/// Send this message when you want to remove paint layers.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Message)]
pub struct TruncatePaintLayers {
    /// The layer index to truncate at.
    layer: LayerIndex,
}

impl TruncatePaintLayers {
    /// Constructs a new [`TruncatePaintLayers`], however, layer 0 is invalid (the length of
    /// histories must always be 1+), so it is incremented.
    pub fn new(LayerIndex(layer_index): LayerIndex) -> Self {
        let layer_index = layer_index.max(1);

        TruncatePaintLayers {
            layer: LayerIndex(layer_index),
        }
    }

    /// Returns an immutable reference to the internal `LayerIndex` value.
    pub fn layer(&self) -> &LayerIndex {
        &self.layer
    }
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
#[require(Name = "HistoryUnit", PaintableHistory::<HistoryUnit> { history: vec![Some(HistoryUnit)] })]
pub struct HistoryUnit;

/// System that returns the last layer index in the history. Pipe this into a system
/// `.after(RecordPaintLayerHistorySet)` to respond to [`RecordPresent`] events if you need to know
/// the newest layer index.
///
/// Previously, this information was stored as a resource, but I found it concerning dealing with
/// the same "indexing" state in two different places that could potentially get out of sync if
/// you're not really careful about scheduling..
pub fn last_layer_index(paintable_history: Single<&PaintableHistory<HistoryUnit>>) -> LayerIndex {
    paintable_history.last_layer_index().unwrap()
}

pub fn triggerable_last_layer_index<E: Event>(
    _: On<E>,
    paintable_history: Single<&PaintableHistory<HistoryUnit>>,
) -> LayerIndex {
    last_layer_index(paintable_history)
}
