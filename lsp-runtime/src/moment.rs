use crate::Timestamp;


/// Moment in LSP system is a point of time when the LSP system may 
/// change its state or the measurement may be taken. 
/// This is the type that describes the moment.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Moment{
    timestamp: Timestamp,
    update_flags: u32,
}

impl Moment {
    const UPDATE_FLAGS_SIGNAL: u32 = 0x1;
    const UPDATE_FLAGS_MEASUREMENT: u32 = 0x2;
    pub fn measurement(timestamp: Timestamp) -> Self {
        Moment { timestamp, update_flags: Self::UPDATE_FLAGS_MEASUREMENT}
    }
    pub fn signal_update(timestamp: Timestamp) -> Self {
        Moment { timestamp, update_flags: Self::UPDATE_FLAGS_SIGNAL}
    }
    pub fn should_update_signals(&self) -> bool {
        (self.update_flags & Self::UPDATE_FLAGS_SIGNAL) > 0
    }
    pub fn should_take_measurements(&self) -> bool {
        (self.update_flags & Self::UPDATE_FLAGS_MEASUREMENT) > 0
    }
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    pub fn merge(&self, other: &Self) -> Option<Self> {
        if self.timestamp != other.timestamp {
            return None;
        }
        Some(Self{
            timestamp: self.timestamp,
            update_flags: self.update_flags | other.update_flags
        })
    }
}