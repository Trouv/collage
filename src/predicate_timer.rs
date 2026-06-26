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
    let timer_entity = app.world_mut().spawn(PredicateTimer::Waiting).id();

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

/// Component that stores the predicate timer, only exists while the predicate is true.
#[derive(Clone, Debug, PartialEq, Eq, Reflect, Resource, Component)]
#[require(Name = "PredicateTimer")]
pub enum PredicateTimer {
    Ticking(Timer),
    Waiting,
}

fn predicate_timer_transition_system(
    entity: Entity,
    initial_timer: Timer,
) -> impl Fn(In<bool>, Res<Time>) -> QueryEntityMap<&'static PredicateTimer, ComponentSet<PredicateTimer>>
{
    move |In(p), time| {
        let delta = time.delta();
        let initial_timer = initial_timer.clone();
        query_entity_map(entity, move |predicate_timer: &PredicateTimer| {
            let new_value = match (p, predicate_timer) {
                (true, PredicateTimer::Ticking(timer)) => {
                    PredicateTimer::Ticking(timer.clone().tick(delta).clone())
                }
                (true, PredicateTimer::Waiting) => PredicateTimer::Ticking(initial_timer.clone()),
                (false, _) => PredicateTimer::Waiting,
            };

            component_set(new_value)
        })
    }
}

fn predicate_timer_finished_trigger(
    entity: Entity,
) -> impl Fn(Query<&PredicateTimer>) -> AffectMany<RepeatN<CommandTrigger<PredicateTimerFinished>>>
{
    move |query| {
        let times_finished = query
            .get(entity)
            .ok()
            .and_then(|timer| match timer {
                PredicateTimer::Ticking(timer) => Some(timer),
                _ => None,
            })
            .map(|timer| timer.times_finished_this_tick())
            .unwrap_or_default();

        affect_many(repeat_n(
            command_trigger(PredicateTimerFinished { entity }),
            times_finished as usize,
        ))
    }
}
