use std::time::SystemTime;

pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> SystemTime;
}

pub struct SystemTimeClock;

impl Clock for SystemTimeClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[cfg(not(test))]
pub(crate) type DefaultClock = SystemTimeClock;

#[cfg(test)]
pub(crate) type DefaultClock = std::sync::Arc<dyn Clock>;

#[cfg(test)]
pub mod test {
    use std::time::{Duration, SystemTime};

    use parking_lot::Mutex;

    use crate::strategy::clock::Clock;

    #[derive(Debug)]
    pub struct MockClock {
        current_time: Mutex<SystemTime>,
    }

    impl MockClock {
        pub fn new() -> Self {
            let time = SystemTime::now();
            Self {
                current_time: Mutex::new(time),
            }
        }

        pub fn advance(&self, duration: Duration) {
            let mut time = self.current_time.lock();
            *time = *time + duration;
        }
    }

    impl Clock for MockClock {
        fn now(&self) -> SystemTime {
            self.current_time.lock().clone()
        }
    }
}
