use bevy::diagnostic::FrameCount;
use bevy::ecs::message::Message;
use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, Message)]
struct DelayedMessage<M: Message>(M);

#[derive(Clone, PartialEq, Eq, Default, Debug, Resource, Deref, DerefMut)]
struct OldMessageQueue<M: Message>(Vec<(u32, M)>);

fn write_delayed_message_system<M: Message + Clone>(
    delay_frames: u32,
) -> impl Fn(
    Res<OldMessageQueue<M>>,
    Res<FrameCount>,
) -> (
    Vec<MessageWrite<DelayedMessage<M>>>,
    ResSet<OldMessageQueue<M>>,
) {
    move |old_message_queue, frame_count| {
        let (messages_to_write, new_message_queue): (Vec<_>, Vec<_>) = old_message_queue
            .0
            .clone()
            .into_iter()
            .partition(|(original_frame, _)| {
                original_frame.saturating_add(delay_frames) < frame_count.0
            });

        (
            messages_to_write
                .into_iter()
                .map(|(_, message)| message_write(DelayedMessage(message)))
                .collect(),
            res_set(OldMessageQueue(new_message_queue)),
        )
    }
}

fn record_messages<M: Message + Clone>(
    frame_count: Res<FrameCount>,
) -> MessagesReadAnd<M, ResSetWith<OldMessageQueue<M>>> {
    let frame_count = frame_count.0;
    messages_read_and(move |m: &M| {
        let m = m.clone();
        res_set_with(move |old_message_queue: &OldMessageQueue<M>| {
            OldMessageQueue(
                old_message_queue
                    .0
                    .clone()
                    .into_iter()
                    .chain(std::iter::once((frame_count, m)))
                    .collect(),
            )
        })
    })
}
