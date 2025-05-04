use std::{collections::HashMap, convert::Infallible, sync::Arc};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::store::Store;

pub struct MemoryStore<S>(Arc<Mutex<HashMap<String, S>>>);

impl<S> Default for MemoryStore<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> MemoryStore<S> {
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::default())
    }
}

impl<S> Store<S> for MemoryStore<S>
where
    S: Send + Clone + Serialize + for<'a> Deserialize<'a>,
{
    type Error = Infallible;

    async fn get(&self, key: &str) -> Result<Option<S>, Self::Error> {
        Ok(self.0.lock().get(key).cloned())
    }

    async fn set(&self, key: &str, state: S) -> Result<(), Self::Error> {
        self.0.lock().insert(key.to_string(), state);
        Ok(())
    }

    async fn clear(&self, key: &str) -> Result<(), Self::Error> {
        self.0.lock().remove(key);
        Ok(())
    }
}
