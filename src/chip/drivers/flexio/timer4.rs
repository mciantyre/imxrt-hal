//! Included when there's 4 timers available.

/// A timer index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Timer {
    /// Timer 0.
    Timer0 = 0,
    /// Timer 1.
    Timer1 = 1,
    /// Timer 2.
    Timer2 = 2,
    /// Timer 3.
    Timer3 = 3,
}

bitflags::bitflags! {
    /// Bitmask for timers.
    pub struct TimerMask: u32 {
        /// Timer 0.
        const TIMER0 = 1 << 0;
        /// Timer 1.
        const TIMER1 = 1 << 1;
        /// Timer 2.
        const TIMER2 = 1 << 2;
        /// Timer 3.
        const TIMER3 = 1 << 3;
    }
}
