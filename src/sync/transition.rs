pub type TransitionCondition<Context, StateKey> =
    fn(&mut Context, &Option<StateKey>, u128) -> Option<StateKey>;
