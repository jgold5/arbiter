use serde::{Serialize, Deserialize};
use std::cell::Cell;

/// Represents an HLC to generate timestamps that are both monotonic and follow real time.
#[derive(Debug, Clone, Copy)]
pub struct HybridLogicalClock<C: WallClock> {
    /// The wall clock component representing real time.
    wall: u64,
    /// The logical component that increments with each event.
    counter: u64,
    clock: C,
}

impl<C:WallClock> HybridLogicalClock<C> {
    /// Initializes the HLC.
    pub fn new(clock: C) -> HybridLogicalClock<C> {
        let start = clock.now_nanos();
        HybridLogicalClock { wall: start, counter: 0, clock}
    }

    /// Creates a new timestamp.
    pub fn tick(&mut self) -> HlcTimestamp {
        let now = self.clock.now_nanos();
        if now > self.wall {
            self.wall = now;
            self.counter = 0;
        // If multiple events occur at the same times (within the same nanosecond), or if an event occurs during an NTP correction
        // order sequencing will be preserved
        } else {
            self.counter += 1;
        }
        HlcTimestamp { wall: self.wall, counter: self.counter }
    }

    pub fn clock(&self) -> &C {
        &self.clock
    }
}

/// Representation of a timestamp produced by the HLC.
/// Uniqueness is guaranteed on a single node as either wall clock or counter is advanced on each call to [`HybridLogicalClock::tick`]
#[derive(Ord, PartialOrd, Eq, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct HlcTimestamp {
    /// The wall clock component representing real time
    wall: u64,
    /// The logical component that increments with each event
    counter: u64,
}

pub struct SystemWallClock;
pub struct TestClock {test_time: Cell<u64>}

pub trait WallClock {
    fn now_nanos(&self) -> u64;
}

impl WallClock for SystemWallClock {
    fn now_nanos(&self) -> u64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64
    }
}

impl WallClock for TestClock {
    fn now_nanos(&self) -> u64 {
        self.test_time.get()
    }
}

impl TestClock {
    pub fn set_time(&self, val: u64) {
       self.test_time.set(val); 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_advances_with_wall_clock() {
        let test_clock = TestClock {test_time: Cell::new(0)};
        let mut hcl = HybridLogicalClock::new(test_clock);
        hcl.clock().set_time(1);
        let now = hcl.tick();
        hcl.clock().set_time(5);
        let after = hcl.tick();
        assert!(after > now);
        assert_eq!(now.counter, 0);
        assert_eq!(after.counter, 0);
    }

    #[test]
    fn test_counter_increments_with_concurrent_wall_time_events() {
        let test_clock = TestClock {test_time: Cell::new(0)};
        let mut hcl = HybridLogicalClock::new(test_clock);
        let first = hcl.tick();
        let next = hcl.tick();
        let last = hcl.tick();
        assert_eq!(first.counter, 1);
        assert_eq!(next.counter, 2);
        assert_eq!(last.counter, 3);
        assert!(last > first);
    }
}