pub mod machine {
    use core::time::Duration;

    use heapless::{FnvIndexMap, Vec};

    use crate::{
        sync::{state::State, transition::TransitionCondition},
        StateKey, StateMachineError, StateMachineSetupResult,
    };

    type DynState<'a, Context, Key> = &'a mut (dyn State<Context, Key> + 'static);

    pub struct StateMachine<'a, Context, Key>
    where
        Key: StateKey + hash32::Hash,
    {
        initial_state: Key,
        current_state: Key,
        transitions: FnvIndexMap<
            Key,
            Vec<TransitionCondition<Context, Key>, heapless::consts::U32>,
            heapless::consts::U32,
        >,
        states: FnvIndexMap<Key, DynState<'a, Context, Key>, heapless::consts::U32>,
        current_state_start_time: u128,
        first_tick: bool,
        user_requested_state: Option<Key>,
    }

    impl<'a, Context, Key> StateMachine<'a, Context, Key>
    where
        Key: StateKey + hash32::Hash,
    {
        pub fn new_state_machine(
            initial_state: DynState<'a, Context, Key>,
        ) -> StateMachine<'a, Context, Key> {
            let mut states: FnvIndexMap<Key, DynState<'a, Context, Key>, heapless::consts::U32> =
                FnvIndexMap::default();

            let state_key = initial_state.state_key();

            if states.insert(state_key, initial_state).is_err() {
                panic!("State machine creation failed pushing initial state in to what should be an empty pre-allocated buffer. Something is very wrong.");
            }

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
            self.user_requested_state.replace(requested_state);
        }

        pub fn add_state(
            &mut self,
            state_to_add: DynState<'a, Context, Key>,
        ) -> StateMachineSetupResult<(), Key> {
            let key = state_to_add.state_key();

            if !self.states.contains_key(&key) {
                if self.states.insert(key, state_to_add).is_err() {
                    return Err(StateMachineError::StackBufferFull);
                }

                Ok(())
            } else {
                Err(StateMachineError::StateAlreadyRegistered(key))
            }
        }

        pub fn current_state(&self) -> Key {
            self.current_state
        }

        pub fn add_transition_condition(
            &mut self,
            from: Key,
            transition: TransitionCondition<Context, Key>,
        ) -> StateMachineSetupResult<(), Key> {
            if self.states.contains_key(&from) {
                if !self.transitions.contains_key(&from)
                    && self.transitions.insert(from, Vec::new()).is_err()
                {
                    return Err(StateMachineError::StackAllocationError);
                }

                let transitions = self.transitions.get_mut(&from).unwrap();
                if transitions.push(transition).is_err() {
                    return Err(StateMachineError::StackAllocationError);
                }

                Ok(())
            } else {
                Err(StateMachineError::TransitionStartStateNotRegistered(from))
            }
        }

        pub fn add_transition_condition_bulk(
            &mut self,
            from: &[Key],
            transition: TransitionCondition<Context, Key>,
        ) -> StateMachineSetupResult<(), Key> {
            for from in from.iter() {
                self.add_transition_condition(*from, transition)?;
            }

            Ok(())
        }

        pub fn check_transition_and_do_action(&mut self, context: &mut Context, time_nanos: u128) {
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
