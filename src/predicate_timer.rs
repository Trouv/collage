use core::iter::{RepeatN, repeat_n};

use bevy::prelude::*;
use bevy_pipe_affect::prelude::*;

/// Add "plugin" for creating and ticking timers while a predicate is true
///
/// Returns the "timer entity" that will be used to store the timer and trigger events.
pub fn add_predicate_timer<P, M>(app: &mut App, initial_timer: Timer, predicate_system: P) -> Entity
where
    P: SystemParamFunction<M, In = (), Out = bool>,
    M: Send + Sync + 'static,
{
    let timer_entity = app.world_mut().spawn(PredicateTimerEntity).id();

    app.add_systems(
        Update,
        (
            predicate_system
                .pipe(predicate_timer_transition_system(
                    timer_entity,
                    initial_timer,
                ))
                .pipe(affect),
            predicate_timer_finished_trigger(timer_entity).pipe(affect),
        )
            .chain(),
    );

    timer_entity
}

/// Event that is triggered when a predicate timer is finished.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Reflect, EntityEvent)]
pub struct PredicateTimerFinished {
    /// The timer entity that finished.
    pub entity: Entity,
}

/// Marker for all predicate timers, regardless of if they are currently timing.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Reflect, Component)]
#[require(Name = "PredicateTimerEntity")]
pub struct PredicateTimerEntity;

/// Component that stores the predicate timer, only exists while the predicate is true.
#[derive(Clone, Default, Debug, PartialEq, Eq, Reflect, Resource, Component, Deref, DerefMut)]
pub struct PredicateTimer(Timer);

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
