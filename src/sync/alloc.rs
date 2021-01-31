pub mod machine {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use core::time::Duration;
    use std::collections::{hash_map::Entry, HashMap};

    use crate::{
        sync::{
            state::{State, StateKey},
            timing::MonotonicTimestamp,
            transition::TransitionCondition,
        },
        StateMachineError, StateMachineSetupResult,
    };

    pub struct StateMachine<Context, Key>
    where
        Key: StateKey,
    {
        initial_state: Key,
        current_state: Key,
        transitions: HashMap<Key, Vec<TransitionCondition<Context, Key>>, ahash::RandomState>,
        states: HashMap<Key, Box<dyn State<Context, Key>>, ahash::RandomState>,
        current_state_start_time: MonotonicTimestamp,
        first_tick: bool,
        user_requested_state: Option<Key>,
    }

    impl<Context, Key> StateMachine<Context, Key>
    where
        Key: StateKey,
    {
        pub fn new_state_machine<S>(initial_state: S) -> StateMachine<Context, Key>
        where
            S: State<Context, Key> + 'static,
        {
            let mut states: HashMap<Key, Box<dyn State<Context, Key>>, ahash::RandomState> =
                HashMap::default();

            let state_key = initial_state.state_key();
            states.insert(state_key, Box::new(initial_state));

            let current_state_start_time = 0;
            let current_state = state_key;

            StateMachine {
                initial_state: state_key,
                current_state,
                transitions: Default::default(),
                states,
                current_state_start_time,
                first_tick: true,
                user_requested_state: None,
            }
        }

        pub fn reset(&mut self, context: &mut Context, time_nanos: u128) {
            let current_state = self.states.get_mut(&self.current_state).unwrap();
            let time_in_state =
                Duration::from_nanos((time_nanos - self.current_state_start_time) as u64);

            current_state.on_exit(context, time_nanos, time_in_state);

            self.current_state_start_time = time_nanos;
            self.current_state = self.initial_state;
            self.first_tick = true;
        }

        pub fn request_transition_from_user(&mut self, requested_state: Key) {
            self.user_requested_state = Some(requested_state);
        }

        pub fn add_state<S>(&mut self, state_to_add: S) -> StateMachineSetupResult<()>
        where
            S: State<Context, Key> + 'static,
        {
            let key = state_to_add.state_key();

            match self.states.entry(key) {
                Entry::Occupied(_) => Err(StateMachineError::StateAlreadyRegistered),
                Entry::Vacant(entry) => {
                    entry.insert(Box::new(state_to_add));
                    Ok(())
                }
            }
        }

        pub fn add_transition_condition_bulk(
            &mut self,
            from: &[Key],
            transition: TransitionCondition<Context, Key>,
        ) -> StateMachineSetupResult<()> {
            for from in from.iter() {
                self.add_transition_condition(*from, transition)?;
            }

            Ok(())
        }

        pub fn add_transition_condition(
            &mut self,
            from: Key,
            transition: TransitionCondition<Context, Key>,
        ) -> StateMachineSetupResult<()> {
            if self.states.contains_key(&from) {
                let transitions = self
                    .transitions
                    .entry(from)
                    .or_insert_with(|| Vec::with_capacity(4));
                transitions.push(transition);
                Ok(())
            } else {
                Err(StateMachineError::TransitionStartStateNotRegistered)
            }
        }

        pub fn current_state(&self) -> Key {
            self.states.get(&self.current_state).unwrap().state_key()
        }

        pub fn check_transition_and_do_action(
            &mut self,
            context: &mut Context,
            time_nanos: MonotonicTimestamp,
        ) {
            let mut current_state = self.states.get_mut(&self.current_state).unwrap();
            let mut time_in_state =
                Duration::from_nanos((time_nanos - self.current_state_start_time) as u64);

            if self.first_tick {
                current_state.on_enter(context, time_nanos);
                self.first_tick = false;
            }

            let requested_state = self.user_requested_state.take();

            if let Some(transitions) = self.transitions.get_mut(&self.current_state) {
                for transition in transitions.iter_mut() {
                    if let Some(new_state) =
                        (transition)(context, &requested_state, time_in_state.as_nanos())
                    {
                        current_state.on_exit(context, time_nanos, time_in_state);

                        self.current_state_start_time = time_nanos;
                        self.current_state = new_state;

                        current_state = self.states.get_mut(&new_state).unwrap();
                        time_in_state = Duration::from_nanos(0);

                        current_state.on_enter(context, time_nanos);

                        break;
                    }
                }
            }

            current_state.do_state_action(context, time_nanos, time_in_state);
        }
    }
}
