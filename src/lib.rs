#![deny(warnings)]
// Declaring our library as `no-std` unconditionally lets us be consistent
// in how we `use` items from `std` or `core`
#![no_std]

// We always pull in `std` during tests, because it's just easier
// to write tests when you can assume you're on a capable platform
#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

#[cfg(any(feature = "std", test))]
#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

// When we're building for a no-std target, we pull in `core`, but alias
// it as `std` so the `use` statements are the same between `std` and `core`.
#[cfg(all(not(feature = "std"), not(test)))]
#[macro_use]
extern crate core as std;

use thiserror::Error;

pub trait StateKey: AsRef<str> + Copy + PartialEq + Eq + std::hash::Hash + std::fmt::Debug {}

impl<T> StateKey for T where
    T: AsRef<str> + Copy + PartialEq + Eq + std::hash::Hash + std::fmt::Debug
{
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Error)]
pub enum StateMachineError<Key>
where
    Key: StateKey,
{
    #[error("Tried to add state for key {0:?} which is already part of the state machine")]
    StateAlreadyRegistered(Key),
    #[error("Tried to add transition with start state key {0:?} which has not been added to the state machine yet")]
    TransitionStartStateNotRegistered(Key),
    #[error("Problem allocating a buffer on the stack")]
    StackAllocationError,
    #[error("Tried to add to a stack buffer that is full")]
    StackBufferFull,
}

pub type StateMachineSetupResult<T, K> = std::result::Result<T, StateMachineError<K>>;

pub mod sync;
