use core::iter::{RepeatN, repeat_n};
use core::marker::PhantomData;

use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

/// Plugin for creating and ticking timers while a predicate is true
#[derive(Clone, Debug, PartialEq, Eq, Reflect, Message)]
struct PredicateTimerPlugin<P, M>
where
    P: SystemParamFunction<M, In = (), Out = bool>,
{
    /// The initial value of a timer when the predicate switches to true.
    initial_timer: Timer,
    /// The entity who the timer will belong, and that will be targeted by
    /// [`PredicateTimerFinished`].
    timer_entity: Entity,
    /// The system returning bool that will be the predicate.
    predicate_system: P,
    marker: PhantomData<M>,
}

pub fn app_with_predicate_timer_plugin<P, M>(
    mut app: App,
    initial_timer: Timer,
    predicate_system: P,
) -> (App, Entity)
where
    P: SystemParamFunction<M, In = (), Out = bool> + Clone,
    M: Send + Sync + 'static,
{
    let timer_entity = app.world_mut().spawn(PredicateTimerEntity).id();

    app.add_plugins(PredicateTimerPlugin {
        initial_timer,
        timer_entity,
        predicate_system,
        marker: PhantomData,
    });

    (app, timer_entity)
}

impl<P, M> Plugin for PredicateTimerPlugin<P, M>
where
    P: SystemParamFunction<M, In = (), Out = bool> + Clone,
    M: Sync + Send + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                self.predicate_system
                    .clone()
                    .pipe(predicate_timer_transition_system(
                        self.timer_entity,
                        self.initial_timer.clone(),
                    ))
                    .pipe(affect),
                predicate_timer_finished_trigger(self.timer_entity).pipe(affect),
            )
                .chain(),
        );
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Reflect, EntityEvent)]
struct PredicateTimerFinished {
    entity: Entity,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Reflect, Component)]
#[require(Name = "PredicateTimerEntity")]
struct PredicateTimerEntity;

#[derive(Clone, Default, Debug, PartialEq, Eq, Reflect, Resource, Component, Deref, DerefMut)]
struct PredicateTimer(Timer);

#[derive(Clone, PartialEq, Eq, Debug, Effect)]
enum PredicateTimerTransition {
    Tick(QueryEntityAffect<ComponentSet<PredicateTimer>>),
    Start(EntityCommandInsert<PredicateTimer>),
    Stop(EntityCommandRemove<PredicateTimer>),
    Wait,
}

fn predicate_timer_transition_system(
    entity: Entity,
    initial_timer: Timer,
) -> impl Fn(In<bool>, Query<&PredicateTimer>, Res<Time>) -> PredicateTimerTransition {
    move |In(p), predicate_timers, time| match (p, predicate_timers.get(entity)) {
        (true, Ok(timer)) => PredicateTimerTransition::Tick(query_entity_affect(
            entity,
            component_set(PredicateTimer(timer.clone().tick(time.delta()).clone())),
        )),
        (true, Err(_)) => PredicateTimerTransition::Start(entity_command_insert(
            entity,
            PredicateTimer(initial_timer.clone()),
        )),
        (false, Ok(_)) => PredicateTimerTransition::Stop(entity_command_remove(entity)),
        (false, _) => PredicateTimerTransition::Wait,
    }
}

fn predicate_timer_finished_trigger(
    entity: Entity,
) -> impl Fn(Query<&PredicateTimer>) -> AffectMany<RepeatN<CommandTrigger<PredicateTimerFinished>>>
{
    move |query| {
        let times_finished = query
            .get(entity)
            .map(|timer| timer.times_finished_this_tick())
            .unwrap_or_default();

        affect_many(repeat_n(
            command_trigger(PredicateTimerFinished { entity }),
            times_finished as usize,
        ))
    }
}
