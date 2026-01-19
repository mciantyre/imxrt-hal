//! Included when there's 8 shifters available.

/// A shift register index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Shifter {
    /// Shifter 0.
    Shifter0 = 0,
    /// Shifter 1.
    Shifter1 = 1,
    /// Shifter 2.
    Shifter2 = 2,
    /// Shifter 3.
    Shifter3 = 3,
    /// Shifter 4.
    Shifter4 = 4,
    /// Shifter 5.
    Shifter5 = 5,
    /// Shifter 6.
    Shifter6 = 6,
    /// Shifter 7.
    Shifter7 = 7,
}

bitflags::bitflags! {
    /// Bitmask for shifters.
    ///
    /// Used for status, error flags, and interrupt / DMA control.
    pub struct ShifterMask: u32 {
        /// Shifter 0.
        const SHIFTER0 = 1 << 0;
        /// Shifter 1.
        const SHIFTER1 = 1 << 1;
        /// Shifter 2.
        const SHIFTER2 = 1 << 2;
        /// Shifter 3.
        const SHIFTER3 = 1 << 3;
        /// Shifter 4.
        const SHIFTER4 = 1 << 4;
        /// Shifter 5.
        const SHIFTER5 = 1 << 5;
        /// Shifter 6.
        const SHIFTER6 = 1 << 6;
        /// Shifter 7.
        const SHIFTER7 = 1 << 7;
    }
}
