//! Included when there's 8 timers available.

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
    /// Timer 4.
    Timer4 = 4,
    /// Timer 5.
    Timer5 = 5,
    /// Timer 6.
    Timer6 = 6,
    /// Timer 7.
    Timer7 = 7,
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
        /// Timer 4.
        const TIMER4 = 1 << 4;
        /// Timer 5.
        const TIMER5 = 1 << 5;
        /// Timer 6.
        const TIMER6 = 1 << 6;
        /// Timer 7.
        const TIMER7 = 1 << 7;
    }
}
