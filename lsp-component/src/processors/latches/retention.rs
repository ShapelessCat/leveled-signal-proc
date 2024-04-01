use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod test {
    use crate::processors::latches::retention::{KeepForever, Retention, TimeToLive};

    #[test]
    fn test_keep_forever() {
        let mut keep_forever = KeepForever;
        assert_eq!(
            <KeepForever as Retention<bool>>::should_drop(&mut keep_forever, 0u64),
            None
        );
        assert_eq!(
            <KeepForever as Retention<bool>>::drop_timestamp(&mut keep_forever, 0u64),
            None
        );
    }

    #[test]
    fn test_time_to_live() {
        let mut time_to_live = TimeToLive {
            default_value: 0,
            value_forgotten_timestamp: 0,
            time_to_live: 2,
        };
        assert_eq!(time_to_live.should_drop(0), Some(0));
        assert_eq!(time_to_live.should_drop(1), Some(0));

        assert_eq!(time_to_live.drop_timestamp(2), Some(2));
        assert_eq!(time_to_live.should_drop(2), None);
        assert_eq!(time_to_live.should_drop(3), None);
        assert_eq!(time_to_live.should_drop(4), Some(0));

        assert_eq!(time_to_live.drop_timestamp(4), Some(2));
        assert_eq!(time_to_live.should_drop(5), None);
        assert_eq!(time_to_live.should_drop(6), Some(0));
        assert_eq!(time_to_live.should_drop(7), Some(0));
    }
}
