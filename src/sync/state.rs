use std::time::Duration;

use crate::StateKey;

pub trait State<Context, Key>
where
    Key: StateKey,
{
    fn state_key(&self) -> Key;

    fn on_enter(&mut self, _context: &mut Context, _timestamp_nanos: u128) {
        if log::log_enabled!(log::Level::Trace) {
            log::trace!("{} on_enter", self.state_key().as_ref());
        }
    }

    fn on_exit(
        &mut self,
        _context: &mut Context,
        _timestamp_nanos: u128,
        _time_in_state: Duration,
    ) {
        if log::log_enabled!(log::Level::Trace) {
            log::trace!("{} on_enter", self.state_key().as_ref());
        }
    }

    fn do_state_action(
        &mut self,
        _context: &mut Context,
        _timestamp_nanos: u128,
        _time_in_state: Duration,
    ) {
        if log::log_enabled!(log::Level::Trace) {
            log::trace!("{} do_state_action", self.state_key().as_ref());
        }
    }
}
