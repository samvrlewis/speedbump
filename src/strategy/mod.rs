use crate::strategy::clock::Clock;
pub(crate) mod clock;

use std::{
    convert::Infallible,
    error::Error,
    fmt::Debug,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::strategy::clock::{DefaultClock, SystemTimeClock};

pub struct LimitResult<M> {
    allowed: bool,
    metadata: Option<M>,
}

impl<M> LimitResult<M> {
    #[must_use]
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            metadata: None,
        }
    }

    #[must_use]
    pub fn disallowed() -> Self {
        Self {
            allowed: false,
            metadata: None,
        }
    }

    #[must_use]
    pub fn with_metadata(self, metadata: M) -> Self {
        Self {
            allowed: self.allowed,
            metadata: Some(metadata),
        }
    }

    pub fn is_allowed(&self) -> bool {
        self.allowed
    }

    pub fn metadata(&self) -> Option<&M> {
        self.metadata.as_ref()
    }
}

pub type DefaultLimitResult = LimitResult<()>;

pub trait LimitStrategy {
    type State: Debug;
    type Error: Error;
    type Metadata: Debug;

    /// Check if a request should be allowed and update the provided state
    ///
    /// # Errors
    /// Errors when a user-defined error occurs in the strategy.
    fn check_limit(
        &self,
        state: &mut Self::State,
    ) -> Result<LimitResult<Self::Metadata>, Self::Error>;

    /// Initialize a new rate limit state for a key
    fn initialize_state(&self) -> Self::State;
}

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
    use std::sync::Arc;

    use super::*;
    use crate::strategy::clock::test::MockClock;

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
