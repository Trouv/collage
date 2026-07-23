use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::predicate_timer::{StartBehavior, add_predicate_timer};

/// Predicate systm that returns `true` while the given button is pressed (on any `ActionState`
/// entity)
pub fn button_predicate<A: Actionlike>(button: A) -> impl Fn(Query<&ActionState<A>>) -> bool {
    move |action_query| action_query.iter().any(|action| action.pressed(&button))
}

/// Add "plugin" for creating and ticking timers while a button is pressed.
pub fn add_button_timer<A: Actionlike>(
    app: &mut App,
    initial_timer: Timer,
    start_behavior: StartBehavior,
    button: A,
) -> Entity {
    add_predicate_timer(app, initial_timer, start_behavior, button_predicate(button))
}
