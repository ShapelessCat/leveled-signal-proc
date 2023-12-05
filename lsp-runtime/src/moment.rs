use crate::Timestamp;

/// Moment in LSP system is a point of time when the LSP system may change its state or the
/// measurement may be taken.
/// This is the type that describes the moment.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Moment {
    timestamp: Timestamp,
    update_flags: u32,
}

impl Moment {
    const UPDATE_FLAGS_SIGNAL: u32 = 0x1;
    const UPDATE_FLAGS_MEASUREMENT: u32 = 0x2;
    #[inline(always)]
    pub fn should_update_group(&self, _group_id: u32) -> bool {
        // Dummy implementation, will be used when the partial update is enabled
        true
    }
    #[inline(always)]
    pub fn measurement(timestamp: Timestamp) -> Self {
        Moment {
            timestamp,
            update_flags: Self::UPDATE_FLAGS_MEASUREMENT,
        }
    }
    #[inline(always)]
    pub fn signal_update(timestamp: Timestamp) -> Self {
        Moment {
            timestamp,
            update_flags: Self::UPDATE_FLAGS_SIGNAL,
        }
    }
    #[inline(always)]
    pub fn should_update_signals(&self) -> bool {
        (self.update_flags & Self::UPDATE_FLAGS_SIGNAL) > 0
    }
    #[inline(always)]
    pub fn should_take_measurements(&self) -> bool {
        (self.update_flags & Self::UPDATE_FLAGS_MEASUREMENT) > 0
    }
    #[inline(always)]
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    pub fn merge(&self, other: &Self) -> Option<Self> {
        if self.timestamp != other.timestamp {
            return None;
        }
        Some(Self {
            timestamp: self.timestamp,
            update_flags: self.update_flags | other.update_flags,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::Moment;

    #[test]
    fn test_merge_moment() {
        let a = Moment::measurement(0);
        let b = Moment::measurement(0);
        let ab = a.merge(&b).unwrap();
        assert!(ab.should_take_measurements());
        assert!(!ab.should_update_signals());
        assert_eq!(ab.timestamp(), 0);

        let a = Moment::measurement(1);
        let b = Moment::signal_update(1);
        let ab = a.merge(&b).unwrap();
        assert!(ab.should_take_measurements());
        assert!(ab.should_update_signals());
        assert_eq!(ab.timestamp(), 1);

        let a = Moment::signal_update(2);
        let b = Moment::signal_update(2);
        let ab = a.merge(&b).unwrap();
        assert!(!ab.should_take_measurements());
        assert!(ab.should_update_signals());
        assert_eq!(ab.timestamp(), 2);

        let a = Moment::signal_update(3);
        let b = Moment::measurement(3);
        let ab = a.merge(&b).unwrap();
        assert!(ab.should_take_measurements());
        assert!(ab.should_update_signals());
        assert_eq!(ab.timestamp(), 3);

        let a = Moment::signal_update(4);
        let b = Moment::measurement(5);
        let ab = a.merge(&b);
        assert!(ab.is_none());
    }
}
