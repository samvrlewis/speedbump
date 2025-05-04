pub(crate) mod clock;
pub mod fixed_window;

use std::{error::Error, fmt::Debug};

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
