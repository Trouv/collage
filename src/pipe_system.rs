//! Taken from bevy docs: <https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemParamFunction.html>,
//! the intention being to create a piped system as a `SystemParamFunction`, which is easier to use
//! in higher-order system logic.
use bevy::ecs::prelude::*;

/// Pipe creates a new system which calls `a`, then calls `b` with the output of `a`
pub fn pipe<A, B, AMarker, BMarker>(
    mut a: A,
    mut b: B,
) -> impl FnMut(ParamSet<(A::Param, B::Param)>) -> B::Out
where
    // We need A and B to be systems, add those bounds
    A: SystemParamFunction<AMarker, In = ()>,
    B: SystemParamFunction<BMarker>,
    for<'a> B::In: SystemInput<Inner<'a> = A::Out>,
{
    // The type of `params` is inferred based on the return of this function above
    move |mut params| {
        let shared = a.run((), params.p0());
        b.run(shared, params.p1())
    }
}
