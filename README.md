# Stately, a simple cyclic synchronous finite state machine framework

There is a lot to be said about state machines in software development. A lot of [really](http://cliffle.com/blog/rust-typestate/) [really](https://hoverbear.org/blog/rust-state-machine-pattern/) [smart](https://blog.yoshuawuyts.com/state-machines/) [people](https://deislabs.io/posts/a-fistful-of-states/) implement them in really smart ways and do really smart things with them.

I am not that smart and neither is this library.

This library is more or less a Rust port (of the design philosophy if not the code itself) of the state machine support classes from [IHMC Open Robotics Software](https://github.com/ihmcrobotics/ihmc-open-robotics-software/blob/1edd95686ad88ada9a9f76f36af6e0d289514a9f/ihmc-robotics-toolkit/src/main/java/us/ihmc/robotics/stateMachine/core/StateMachine.java).

This library exists to fit in the niche of incorporating a relatively relaxed implementation of a state machine as a synchronous component in a cyclic process. The design borrowed here was incubated in the context of embedding state charts in feedback control loops where the state complexity makes it a bit of a PITA to implement the states in a more "common" way like a switch statement over a bunch of enums with function calls for the state actions. In particular, it makes it much much easier to manage transition conditions and cleanup by introducing well defined encapsulation boundaries between individual states as well as between individual transition conditions.

## Why this, not _x_?

There are plenty of generic state machine frameworks out there already; they often do things like encoding transitions in the type system, or using `From`/`Into` to enforce transitions, or having compile-time checked transition completeness. Some of them encode state machines using futures for long running asynchronous operations.

When writing simple single threaded digital feedback loops, though, it's not uncommon to find yourself needing a state-machine-like-construct in the middle of your main loop somewhere that determines what, exactly, the main loop should be doing this iteration. A common design pattern in this case is to encode ones states as a set of enums and then have a switch on top of them. This type of construct is still immensely useful. In Rust, you can even codify the transition conditions by `match`'ing on a tuple of states (current state, desired state) and doing some logic to determine if that pair is a viable transition.

But as soon as your loop actions become non-trivial, and/or your state transitions become non-trivial, and/or your system's shared state becomes non-trivial, and/or you find yourself needing to do a bunch of time-based logic/transitioning, it might be nice to have a reusable piece of software to reach for. Enter `stately`.

Stately **_DOES NOT_** do things like make sure all of your states are reachable or that your state machine terminates. The first it probably could do, the second is not its goal. In fact many of the machines you would encode with this library might never terminate because your loop might never terminate. But that's okay.

What Stately **_DOES_** do is give you a way for writing highly encapsulated states with encapsulated transition conditions. A state takes only its own local state and the shared system state (called the `Context`) as inputs for its actions and the transition conditions similarly take only the `Context` as their inputs for determining if a transition is valid. States are implemented by adhering to traits defined in the crate. Stately also provides some nice tools for the caller to provide time signals to simplify time-based transition and action logic.

## What's up w/ the u128 time arguments for the update functions?

As mentioned above the context in which this design was first established at IHMC was done in the context of digital feedback loops for control systems; these loops are nearly [hard real-time](https://en.wikipedia.org/wiki/Real-time_computing) feedback loops. Timing for these loops when they aren't done bare metal is typically built on top of an OS's high resolution monotonic clock rather than the wall clock. This is because the wall clock can skew backwards or forward in time arbitrarily due to many reasons; user timezone adjustment, daylight saving time, NTP corrections, etc.

High resolution monotonic clocks typically report their values using C-style `timespec`-adjacent structs w/ a Seconds and Nanoseconds field (e.g. `Duration` in Rust), or they use cumulative nanoseconds encoded in an unsigned 64 bit type.

So why `u128`? Because the easiest way to get a high resolution monotonic clock in Rust is via creating a long-lived `Instant` via `std::time::Instant::now()` and then calling `Instant::elapsed().as_nanos()` which returns a `u128`.

## `no_std`

By default, `stately` leverages the Rust standard library. Stately uses dynamic dispatch for both states and their transition conditions. In the default configuration this is achieved by storing `Vec`'s and `HashMap`'s of [`Box`'ed Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html) for the concrete state implementations and their transition conditions in the main state machine struct. All of the relevant implementations for this configuration can be found in the `stately::sync::alloc` module.

Stately supports `no_std` environments if you set `--no-default-features` by leveraging the [`heapless`](https://docs.rs/heapless/0.7.3/heapless/) crate. In this configuration, the State Machine holds references to the states and their transition conditions in these stack allocated collections. Using stately in this way would require for the states and transitions to be defined in such a way that they can be uniquely (mutably) borrowed by the State Machine. All of the relevenat implementations for this configuration can be found in the `stately::sync::heapless` module.