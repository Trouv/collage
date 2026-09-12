use std::marker::PhantomData;

use bevy::diagnostic::{FrameCount, update_frame_count};
use bevy::ecs::message::Message;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

/// Generic plugin that replicates messages after a certain delay.
///
/// Writes a [`DelayedMessage<M, N>`] message `N` frames after `M` was written.
///
/// [`DelayedMessage<M, N>`]: [DelayedMessage]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct DelayMessagePlugin<M: Message, const DELAY_FRAMES: u32> {
    phantom: PhantomData<M>,
}

impl<M: Message + Clone, const DELAY_FRAMES: u32> Plugin for DelayMessagePlugin<M, DELAY_FRAMES> {
    fn build(&self, app: &mut App) {
        app.add_message::<DelayedMessage<M, DELAY_FRAMES>>()
            .insert_resource(OldMessageQueue::<M, DELAY_FRAMES>(vec![]))
            .add_systems(
                Last,
                record_messages::<M, DELAY_FRAMES>
                    .pipe(affect)
                    .before(update_frame_count),
            )
            .add_systems(
                First,
                write_delayed_message_system::<M, DELAY_FRAMES>.pipe(affect),
            );
    }
}

/// The delayed message sent by this plugin.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, Message, Deref, DerefMut)]
pub struct DelayedMessage<M: Message, const DELAY_FRAMES: u32>(pub M);

#[derive(Clone, PartialEq, Eq, Default, Debug, Resource, Deref, DerefMut)]
struct OldMessageQueue<M: Message, const DELAY_FRAMES: u32>(Vec<(u32, M)>);

fn write_delayed_message_system<M: Message + Clone, const DELAY_FRAMES: u32>(
    old_message_queue: Res<OldMessageQueue<M, DELAY_FRAMES>>,
    frame_count: Res<FrameCount>,
) -> (
    Vec<MessageWrite<DelayedMessage<M, DELAY_FRAMES>>>,
    ResSet<OldMessageQueue<M, DELAY_FRAMES>>,
) {
    let (messages_to_write, new_message_queue): (Vec<_>, Vec<_>) = old_message_queue
        .0
        .clone()
        .into_iter()
        .partition(|(original_frame, _)| {
            original_frame.saturating_add(DELAY_FRAMES) < frame_count.0
        });

    (
        messages_to_write
            .into_iter()
            .map(|(_, message)| message_write(DelayedMessage(message)))
            .collect(),
        res_set(OldMessageQueue(new_message_queue)),
    )
}

fn record_messages<M: Message + Clone, const DELAY_FRAMES: u32>(
    frame_count: Res<FrameCount>,
) -> MessagesReadAnd<M, ResSetWith<OldMessageQueue<M, DELAY_FRAMES>>> {
    let frame_count = frame_count.0;
    messages_read_and(move |m: &M| {
        let m = m.clone();
        res_set_with(
            move |old_message_queue: &OldMessageQueue<M, DELAY_FRAMES>| {
                OldMessageQueue(
                    old_message_queue
                        .0
                        .clone()
                        .into_iter()
                        .chain(std::iter::once((frame_count, m)))
                        .collect(),
                )
            },
        )
    })
}
