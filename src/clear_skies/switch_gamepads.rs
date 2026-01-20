use std::marker::PhantomData;

use bevy::input::gamepad::{
    GamepadAxisChangedEvent,
    GamepadButtonChangedEvent,
    GamepadConnectionEvent,
    GamepadEvent,
};
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy_pipe_affect::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SwitchGamepadsPlugin<A: Actionlike + TypePath + GetTypeRegistration>(PhantomData<A>);

impl<A: Actionlike + TypePath + GetTypeRegistration> Default for SwitchGamepadsPlugin<A> {
    fn default() -> Self {
        SwitchGamepadsPlugin(PhantomData)
    }
}

impl<A: Actionlike + TypePath + GetTypeRegistration> Plugin for SwitchGamepadsPlugin<A> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<InputManagerPlugin<A>>() {
            app.add_plugins(InputManagerPlugin::<A>::default());
        }

        app.add_systems(FixedUpdate, switch_gamepads::<A>.pipe(affect));
    }
}

pub fn switch_gamepads<A: Actionlike>(
    mut gamepad_events: MessageReader<GamepadEvent>,
) -> Vec<impl Effect + use<A>> {
    gamepad_events
        .read()
        .map(|event| {
            let (GamepadEvent::Connection(GamepadConnectionEvent {
                gamepad: entity, ..
            })
            | GamepadEvent::Button(GamepadButtonChangedEvent { entity, .. })
            | GamepadEvent::Axis(GamepadAxisChangedEvent { entity, .. })) = event;

            let entity = *entity;

            components_set_with::<_, _>(move |(input_map,): (InputMap<A>,)| {
                (input_map.with_gamepad(entity),)
            })
        })
        .collect()
}
