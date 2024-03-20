use serde::{de::DeserializeOwned, Deserialize, Serialize};

use lsp_runtime::{Duration, Timestamp};

/// Abstracts the retention behavior of a latch
pub trait Retention<T> {
    fn drop_timestamp(&mut self, now: Timestamp) -> Option<Timestamp>;
    fn should_drop(&mut self, now: Timestamp) -> Option<T>;
}

/// The retention policy for latches that keep the value forever
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct KeepForever;

impl<T> Retention<T> for KeepForever {
    fn drop_timestamp(&mut self, _: Timestamp) -> Option<Timestamp> {
        None
    }

    fn should_drop(&mut self, _: Timestamp) -> Option<T> {
        None
    }
}

/// The retention policy for latches that keep the value for a period of time
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeToLive<T> {
    pub(super) default_value: T,
    pub(super) value_forgotten_timestamp: Timestamp,
    pub(super) time_to_live: Duration,
}

impl<T: Clone + Serialize + DeserializeOwned> Retention<T> for TimeToLive<T> {
    fn drop_timestamp(&mut self, now: Timestamp) -> Option<Timestamp> {
        self.value_forgotten_timestamp = now + self.time_to_live;
        Some(self.time_to_live)
    }

    fn should_drop(&mut self, now: Timestamp) -> Option<T> {
        if self.value_forgotten_timestamp <= now {
            Some(self.default_value.clone())
        } else {
            None
        }
    }
}
