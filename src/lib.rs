use crate::{store::LocalStore, strategy::LimitStrategy};

pub mod store;
pub mod strategy;

impl Limiter<Unset, Unset> {
    #[must_use]
    pub fn builder() -> LimiterBuilder<Unset, Unset> {
        LimiterBuilder::new()
    }
}

pub struct Limiter<Store, Strategy> {
    store: Store,
    strategy: Strategy,
}

#[derive(thiserror::Error, Debug)]
pub enum Error<Str, Sto> {
    Strategy(Str),
    Store(Sto),
}

impl<Store, Strategy> Limiter<Store, Strategy>
where
    Strategy: LimitStrategy,
    Store: LocalStore<<Strategy as LimitStrategy>::State>,
{
    /// Checks the limit for a given key
    ///
    /// # Errors
    /// Errors when the underlying strategy or store errors
    pub async fn limit(&self, key: &str) -> Result<bool, Error<Strategy::Error, Store::Error>> {
        let mut state = self
            .store
            .get(key)
            .await
            .map_err(Error::Store)?
            .unwrap_or(self.strategy.initialize_state());

        let limited = self
            .strategy
            .check_limit(&mut state)
            .map_err(Error::Strategy)?;

        self.store.set(key, state).await.map_err(Error::Store)?;

        Ok(limited.is_allowed())
    }
}
pub struct Unset;

pub struct LimiterBuilder<Store, Strategy> {
    store: Store,
    strategy: Strategy,
}

impl Default for LimiterBuilder<Unset, Unset> {
    fn default() -> Self {
        Self::new()
    }
}

impl LimiterBuilder<Unset, Unset> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            store: Unset,
            strategy: Unset,
        }
    }
}

impl<Store, Strategy> LimiterBuilder<Store, Strategy> {
    pub fn store<S>(self, store: S) -> LimiterBuilder<S, Strategy> {
        LimiterBuilder {
            store,
            strategy: self.strategy,
        }
    }

    pub fn strategy<S>(self, strategy: S) -> LimiterBuilder<Store, S> {
        LimiterBuilder {
            store: self.store,
            strategy,
        }
    }
}

impl<Store, Strategy> LimiterBuilder<Store, Strategy>
where
    Store: LocalStore<<Strategy as LimitStrategy>::State>,
    Strategy: LimitStrategy,
{
    pub fn build(self) -> Limiter<Store, Strategy> {
        Limiter {
            store: self.store,
            strategy: self.strategy,
        }
    }
}
