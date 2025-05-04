pub mod memory;

use std::error::Error;

#[trait_variant::make(Store: Send)]
pub trait LocalStore<S> {
    type Error: Send + Sync + Error;

    async fn get(&self, key: &str) -> Result<Option<S>, Self::Error>;

    async fn set(&self, key: &str, state: S) -> Result<(), Self::Error>;

    async fn clear(&self, key: &str) -> Result<(), Self::Error>;
}
