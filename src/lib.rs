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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum StateMachineError {
    StateAlreadyRegistered,
    TransitionStartStateNotRegistered,
    StackAllocationError,
    StackBufferFull,
}

pub type StateMachineSetupResult<T> = std::result::Result<T, StateMachineError>;

pub mod sync;
