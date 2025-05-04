use std::{
    convert::Infallible,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::strategy::{
    LimitResult, LimitStrategy,
    clock::{Clock, DefaultClock, SystemTimeClock},
};

pub struct FixedWindow {
    window_duration: Duration,
    limit: u32,
    clock: DefaultClock,
}

impl FixedWindow {
    #[must_use]
    pub fn new(window_duration: Duration, limit: u32) -> Self {
        #[cfg(not(test))]
        let clock = SystemTimeClock;

        #[cfg(test)]
        let clock = std::sync::Arc::new(SystemTimeClock) as DefaultClock;

        Self {
            window_duration,
            limit,
            clock,
        }
    }

    #[cfg(test)]
    pub fn with_clock(self, clock: DefaultClock) -> Self {
        Self {
            window_duration: self.window_duration,
            limit: self.limit,
            clock,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FixedWindowCounterState {
    count: u32,
    window_start: SystemTime,
}

#[derive(Debug, Clone, Copy)]
pub struct FixedWindowMetadata {
    pub till_next_window: Duration,
}

impl LimitStrategy for FixedWindow {
    type State = FixedWindowCounterState;
    type Error = Infallible;
    type Metadata = FixedWindowMetadata;

    fn check_limit(
        &self,
        state: &mut Self::State,
    ) -> Result<LimitResult<FixedWindowMetadata>, Self::Error> {
        let now = self.clock.now();
        let time_since_start = now
            .duration_since(state.window_start)
            .unwrap_or(Duration::MAX);

        let allowed = if time_since_start < self.window_duration {
            if state.count < self.limit {
                state.count += 1;
                LimitResult::allowed()
            } else {
                LimitResult::disallowed()
            }
        } else {
            // New window, reset the counter and update the start time
            state.count = 1;
            state.window_start = now;
            LimitResult::allowed()
        };

        let next_window = state
            .window_start
            .checked_add(self.window_duration)
            .map_or(Duration::MAX, |x| {
                x.duration_since(now).unwrap_or(Duration::MAX)
            });

        let result = allowed.with_metadata(FixedWindowMetadata {
            till_next_window: next_window,
        });

        Ok(result)
    }

    fn initialize_state(&self) -> Self::State {
        FixedWindowCounterState {
            count: 0,
            window_start: self.clock.now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use crate::strategy::{clock::test::MockClock, fixed_window::FixedWindow, *};

    #[test]
    fn test_fixed_window() {
        let clock = Arc::new(MockClock::new());
        let fixed_window = FixedWindow::new(Duration::from_secs(10), 3).with_clock(clock.clone());
        let mut state = fixed_window.initialize_state();

        // should let 3 through
        assert!(fixed_window.check_limit(&mut state).unwrap().is_allowed());
        assert!(fixed_window.check_limit(&mut state).unwrap().is_allowed());
        assert!(fixed_window.check_limit(&mut state).unwrap().is_allowed());

        // and then stop the next
        assert!(!fixed_window.check_limit(&mut state).unwrap().is_allowed());

        // advancing 10 seconds should allow again
        clock.advance(Duration::from_secs(10));

        // should let a further 3 through
        assert!(fixed_window.check_limit(&mut state).unwrap().is_allowed());
        assert!(fixed_window.check_limit(&mut state).unwrap().is_allowed());
        assert!(fixed_window.check_limit(&mut state).unwrap().is_allowed());

        // and then stop the next
        assert!(!fixed_window.check_limit(&mut state).unwrap().is_allowed());
    }
}
