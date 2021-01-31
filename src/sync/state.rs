use std::time::Duration;

use super::timing::MonotonicTimestamp;

pub trait StateKey:
    AsRef<str> + Copy + PartialEq + Eq + hash32::Hash + std::hash::Hash + std::fmt::Debug
{
}

pub trait State<Context, Key>: std::fmt::Debug
where
    Key: StateKey,
{
    fn state_key(&self) -> Key;

    fn on_enter(&mut self, _context: &mut Context, _timestamp_nanos: MonotonicTimestamp) {
        if log::log_enabled!(log::Level::Trace) {
            log::trace!("{} on_enter", self.state_key().as_ref());
        }
    }

    fn on_exit(
        &mut self,
        _context: &mut Context,
        _timestamp_nanos: MonotonicTimestamp,
        _time_in_state: Duration,
    ) {
        if log::log_enabled!(log::Level::Trace) {
            log::trace!("{} on_enter", self.state_key().as_ref());
        }
    }

    fn do_state_action(
        &mut self,
        _context: &mut Context,
        _timestamp_nanos: MonotonicTimestamp,
        _time_in_state: Duration,
    ) {
        if log::log_enabled!(log::Level::Trace) {
            log::trace!("{} do_state_action", self.state_key().as_ref());
        }
    }
}
