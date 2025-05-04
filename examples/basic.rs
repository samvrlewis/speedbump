use std::time::Duration;

use speedbump::{Limiter, store::memory::MemoryStore, strategy::FixedWindow};

#[tokio::main]
async fn main() {
    let fixed_window = FixedWindow::new(Duration::from_secs(10), 10);

    let store = MemoryStore::new();
    let limiter = Limiter::builder()
        .store(store)
        .strategy(fixed_window)
        .build();

    for iteration in 0..20 {
        let allowed = limiter.limit("test").await.expect("limit checking failed");
        println!("Iteration {:?}, allowed: {}", iteration, allowed);
    }
}
