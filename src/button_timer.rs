use core::marker::PhantomData;

use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;
use leafwing_input_manager::prelude::*;

trait TimeableButton {
    type Action: Actionlike;

    const BUTTON: Self::Action;
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Reflect, Message)]
struct ButtonTimerPlugin<TB>
where
    TB: TimeableButton,
{
    timer: Timer,
    tb: PhantomData<TB>,
}

impl<TB> Plugin for ButtonTimerPlugin<TB>
where
    TB: TimeableButton + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                press_button_timer_system::<TB>(self.timer.clone()).pipe(affect),
                (
                    tick_button_timer::<TB>.pipe(affect),
                    button_timer_finished_message::<TB>.pipe(affect),
                )
                    .chain(),
                release_button_timer::<TB>.pipe(affect),
            ),
        );
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Reflect, Message)]
struct ButtonTimerFinished<TB>
where
    TB: TimeableButton,
{
    entity: Entity,
    tb: PhantomData<TB>,
}

impl<TB> ButtonTimerFinished<TB>
where
    TB: TimeableButton,
{
    fn new(entity: Entity) -> Self {
        ButtonTimerFinished {
            entity,
            tb: PhantomData,
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Reflect, Resource, Component)]
struct ButtonTimer<TB>
where
    TB: TimeableButton,
{
    timer: Timer,
    tb: PhantomData<TB>,
}

impl<TB> ButtonTimer<TB>
where
    TB: TimeableButton,
{
    fn new(timer: Timer) -> Self {
        ButtonTimer {
            timer,
            tb: PhantomData,
        }
    }
}

fn press_button_timer_system<TB>(
    timer: Timer,
) -> impl Fn(Query<(Entity, &ActionState<TB::Action>)>) -> Vec<EntityCommandInsert<ButtonTimer<TB>>>
where
    TB: TimeableButton + Send + Sync + 'static,
{
    move |query| {
        query
            .iter()
            .flat_map(|(entity, action_state)| {
                action_state
                    .just_pressed(&TB::BUTTON)
                    .then(|| entity_command_insert(entity, ButtonTimer::new(timer.clone())))
            })
            .collect()
    }
}

fn tick_button_timer<TB>(
    time: Res<Time>,
) -> QueryMap<&'static ButtonTimer<TB>, ComponentSet<ButtonTimer<TB>>>
where
    TB: TimeableButton + Send + Sync + 'static,
{
    let delta = time.delta();
    query_map(move |button_timer: &ButtonTimer<TB>| {
        component_set(ButtonTimer::<TB>::new(
            button_timer.timer.clone().tick(delta).clone(),
        ))
    })
}

fn button_timer_finished_message<TB>(
    query: Query<(Entity, &ButtonTimer<TB>)>,
) -> Vec<MessageWrite<ButtonTimerFinished<TB>>>
where
    TB: TimeableButton + Send + Sync + 'static,
{
    query
        .iter()
        .filter(|(_, button_timer)| button_timer.timer.just_finished())
        .map(|(entity, _)| message_write(ButtonTimerFinished::new(entity)))
        .collect()
}

fn release_button_timer<TB>(
    query: Query<(Entity, &ActionState<TB::Action>), With<ButtonTimer<TB>>>,
) -> Vec<EntityCommandRemove<ButtonTimer<TB>>>
where
    TB: TimeableButton + Send + Sync + 'static,
{
    query
        .iter()
        .flat_map(|(entity, action_state)| {
            action_state
                .just_released(&TB::BUTTON)
                .then(|| entity_command_remove(entity))
        })
        .collect()
}
