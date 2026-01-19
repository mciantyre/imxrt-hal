//! Flexible I/O.
//!
//! By configuring shift registers and timers, you can drive output pins and
//! read input pins, emulating various protocols. This driver exposes the shift
//! registers and timers.
//!
//! The [`FlexIo`] driver may be shared across different emulated protocol drivers
//! or objects. You're responsible for making sure timers and shifters are not shared
//! across these devices. These are not represented by exclusive objects.

pub use crate::chip::config::flexio::*;

use core::ops::BitOr;

use crate::ral;

/// Any FlexIO instance.
type AnyInstance = crate::AnyInstance<ral::flexio::RegisterBlock>;

impl Shifter {
    /// Returns this shifter's bitmask.
    #[inline]
    #[must_use]
    pub const fn mask(self) -> ShifterMask {
        ShifterMask::from_bits_truncate(1 << (self as u32))
    }
}

impl From<Shifter> for ShifterMask {
    #[inline]
    fn from(shifter: Shifter) -> Self {
        shifter.mask()
    }
}

impl BitOr for Shifter {
    type Output = ShifterMask;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        ShifterMask::from(self) | ShifterMask::from(rhs)
    }
}

impl BitOr<ShifterMask> for Shifter {
    type Output = ShifterMask;
    #[inline]
    fn bitor(self, rhs: ShifterMask) -> Self::Output {
        ShifterMask::from(self) | rhs
    }
}

impl BitOr<Shifter> for ShifterMask {
    type Output = ShifterMask;
    #[inline]
    fn bitor(self, rhs: Shifter) -> Self::Output {
        self | ShifterMask::from(rhs)
    }
}

impl Timer {
    /// Returns this timer's bitmask.
    #[inline]
    #[must_use]
    pub const fn mask(self) -> TimerMask {
        TimerMask::from_bits_truncate(1 << (self as u32))
    }

    /// Returns `true` if `fst` is the one right before `snd`.
    ///
    /// Order matters. `fst` must be one below `snd`.
    ///
    /// ```
    /// use imxrt_hal::flexio::Timer;
    ///
    /// assert!(Timer::are_adjacent(Timer::Timer1, Timer::Timer2));
    /// assert!(!Timer::are_adjacent(Timer::Timer2, Timer::Timer1));
    /// assert!(!Timer::are_adjacent(Timer::Timer1, Timer::Timer1));
    /// assert!(Timer::are_adjacent(Timer::Timer6, Timer::Timer7));
    /// assert!(Timer::are_adjacent(Timer::Timer0, Timer::Timer1));
    /// ```
    #[inline]
    #[must_use]
    pub const fn are_adjacent(fst: Self, snd: Self) -> bool {
        (fst as usize + 1) == snd as usize
    }
}

impl From<Timer> for TimerMask {
    #[inline]
    fn from(timer: Timer) -> Self {
        timer.mask()
    }
}

impl BitOr for Timer {
    type Output = TimerMask;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        TimerMask::from(self) | TimerMask::from(rhs)
    }
}

impl BitOr<TimerMask> for Timer {
    type Output = TimerMask;
    #[inline]
    fn bitor(self, rhs: TimerMask) -> Self::Output {
        TimerMask::from(self) | rhs
    }
}

impl BitOr<Timer> for TimerMask {
    type Output = TimerMask;
    #[inline]
    fn bitor(self, rhs: Timer) -> Self::Output {
        self | TimerMask::from(rhs)
    }
}

/// Shifter operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum ShifterMode {
    /// Receive mode.
    Receive = 0b001,
    /// Transmit mode.
    Transmit = 0b010,
    /// Match Store mode.
    MatchStore = 0b100,
    /// Match Continuous mode.
    MatchContinuous = 0b101,
    /// State mode.
    State = 0b110,
    /// Logic mode.
    Logic = 0b111,
}

/// Shifter input source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum ShifterInput {
    /// Input from pin.
    Pin = 0,
    /// Input from next shifter's output.
    Shifter = 1,
}

/// Which timer edge performs a shift?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum ShifterTimerPolarity {
    /// Shift on positive edge of timer.
    PositiveEdge = 0,
    /// Shift on negative edge of timer.
    NegativeEdge = 1,
}

impl ShifterTimerPolarity {
    /// Flip the polarity.
    #[inline]
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::PositiveEdge => Self::NegativeEdge,
            Self::NegativeEdge => Self::PositiveEdge,
        }
    }
}

/// Pin configuration for shifters and timers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum PinConfig {
    /// Output disabled.
    ///
    /// It's an input.
    OutputDisabled = 0b00,
    /// Output enable.
    ///
    /// Can be used as an open drain, assuming inverted logic.
    /// Could also be used to control an "enable" signal for
    /// bidirectional data; use another timer / shifter's
    /// output to (un)mask bi-directional output data.
    OutputEnable = 0b01,
    /// Bidirectional output data.
    Bidirectional = 0b10,
    /// Output pin.
    Output = 0b11,
}

/// Output pin polarity for shifters and timers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum PinPolarity {
    /// Active high.
    ActiveHigh = 0,
    /// Active low (inverted).
    ActiveLow = 1,
}

/// Configurations for shifter and timer pins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PinSelection {
    /// Which pin to use as an input / output.
    pub select: Pin,
    /// How that pin is configured.
    pub config: PinConfig,
    /// The pin's input / output polarity.
    pub polarity: PinPolarity,
}

impl PinSelection {
    /// There is no pin selection.
    ///
    /// If this shifter is chained to another
    /// shifter, then it may not require a
    /// pin selection.
    pub const NONE: Self = Self {
        select: Pin(0),
        config: PinConfig::OutputDisabled,
        polarity: PinPolarity::ActiveHigh,
    };
}

/// Shifter start bit configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum ShifterStartBit {
    /// Start bit disabled, transmitter loads data on enable.
    DisabledLoadOnEnable = 0b00,
    /// Start bit disabled, transmitter loads data on first shift.
    DisabledLoadOnFirstShift = 0b01,
    /// Transmitter outputs start bit value 0 before loading data.
    ///
    /// Receiver/match store sets error flag if start bit is not 0.
    StartBitZero = 0b10,
    /// Transmitter outputs start bit value 1 before loading data.
    ///
    /// Receiver/match store sets error flag if start bit is not 1.
    StartBitOne = 0b11,
}

/// Shifter stop bit configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum ShifterStopBit {
    /// Stop bit disabled.
    Disabled = 0b00,
    /// Transmitter outputs stop bit value 0 on store.
    ///
    /// Receiver/match store sets error flag if stop bit is not 0.
    StopBitZero = 0b10,
    /// Transmitter outputs stop bit value 1 on store.
    ///
    /// Receiver/match store sets error flag if stop bit is not 1.
    StopBitOne = 0b11,
}

/// Shifter configuration.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ShifterConfig {
    /// The enabled shifter mode.
    pub mode: ShifterMode,
    /// Which timer controls this shifter.
    pub timer: Timer,
    /// Shifter timer polarity.
    pub timer_polarity: ShifterTimerPolarity,
    /// Configurations for a pin, if enabled.
    pub pin: PinSelection,
    /// Shifter input source.
    pub input_source: ShifterInput,
    /// Start bit configuration.
    pub start_bit: ShifterStartBit,
    /// Stop bit configuration.
    pub stop_bit: ShifterStopBit,
    /// Parallel width, or disable state outputs in state mode.
    ///
    /// See the reference manual for more information. The
    /// implementation writes this value directly to `PWIDTH`.
    ///
    /// For serial transmit / receive, set this to 0.
    pub parallel_width: u8,
}

/// Timer operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
#[non_exhaustive]
pub enum TimerMode {
    /// Dual 8-bit counters baud mode.
    ///
    /// The lower 8 bits of the compare register specify the bit
    /// rate of the shift clock, and the upper 8 bits specify the
    /// number of shift clock edges. When the lower 8 bits reach zero,
    /// the upper 8 bits decrement, and the lower bits reset from
    /// the compare register. The timer compares when it wraps around
    /// from the all-zero bit pattern.
    ///
    /// Use the lower 8 bits to introduce a divider on the input
    /// clock. Use the upper 8 bits to control how many bits are shifted.
    ///
    /// Note that a timer reset only affects the lower 8 bits.
    DualBaudCounter = 0b01,
    /// Dual 8-bit counters PWM high mode.
    ///
    /// Lower 8 bits configure a period for driving a high output
    /// from the timer. The upper 8 bits configures a period for driving
    /// a low output. Remember: low byte drives high, and high byte drives
    /// low.
    ///
    /// First, the low byte decrements. When it wraps around zero, the output
    /// flips to low, the low byte reloads, and the high byte starts
    /// decrementing. When that reaches wraps from zero, the high byte
    /// reloads, and a compare event is generated.
    ///
    /// Useful for realizing a PWM signal.
    DualPwmHigh = 0b10,
    /// Single 16-bit counter.
    ///
    /// Decrement the counter, starting from the compare register's value,
    /// based on the [`TimerDecrement`] selection. When it wraps around from
    /// zero, generate a compare event.
    SingleCounter = 0b11,
}

/// Timer trigger source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
enum TimerTriggerSource {
    /// External trigger.
    External = 0,
    /// Internal trigger (from pin or shifter).
    Internal = 1,
}

/// Timer trigger polarity.
///
/// Can be used to invert the [`TimerTrigger`] input signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TimerTriggerPolarity {
    /// Trigger active high.
    ActiveHigh = 0,
    /// Trigger active low.
    ActiveLow = 1,
}

/// Timer trigger select.
///
/// Selects which pin, shifter, or timer generates the trigger signal.
/// Use the constructor methods to create a trigger select value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct TimerTriggerSelect(u8);

impl TimerTriggerSelect {
    /// Create a trigger select from a pin index.
    #[inline]
    #[must_use]
    const fn from_pin(pin: Pin) -> Self {
        Self(2 * pin.0)
    }

    /// Create a trigger select from a shifter.
    #[inline]
    #[must_use]
    const fn from_shifter(shifter: Shifter) -> Self {
        Self(4 * (shifter as u8) + 1)
    }

    /// Create a trigger select from a timer.
    #[inline]
    #[must_use]
    const fn from_timer(timer: Timer) -> Self {
        Self(4 * (timer as u8) + 3)
    }
}

/// Timer triggers, either internal or external.
///
/// Internal triggers come from pins, shifters, or other
/// timers. External triggers can be crossbar signals, or
/// other inputs to the IP block.
///
/// The trigger is dependent on the [`TimerTriggerPolarity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimerTrigger {
    /// Which internal / external trigger?
    select: TimerTriggerSelect,
    /// Is it internal or external?
    source: TimerTriggerSource,
}

impl TimerTrigger {
    /// The trigger is internal.
    #[inline]
    #[must_use]
    const fn internal(select: TimerTriggerSelect) -> Self {
        Self {
            select,
            source: TimerTriggerSource::Internal,
        }
    }

    /// Create a trigger select from a pin index.
    ///
    /// The trigger will be the input value of the specified pin.
    #[inline]
    #[must_use]
    pub const fn from_pin(pin: Pin) -> Self {
        Self::internal(TimerTriggerSelect::from_pin(pin))
    }

    /// Trigger from the given `timer`.
    ///
    /// The trigger will be the output of the specified timer.
    /// This is useful for chaining timers together.
    #[inline]
    #[must_use]
    pub const fn from_timer(timer: Timer) -> Self {
        Self::internal(TimerTriggerSelect::from_timer(timer))
    }

    /// Trigger from the given `shifter`.
    ///
    /// The trigger will be the status flag of the specified shifter.
    /// This is useful for chaining shifter operations with timer control.
    #[inline]
    #[must_use]
    pub const fn from_shifter(shifter: Shifter) -> Self {
        Self::internal(TimerTriggerSelect::from_shifter(shifter))
    }

    /// Trigger from an external `signal`.
    ///
    /// The valid values, and the signal source depends on your specific
    /// FlexIO implementation. This value is passed directly into the
    /// register with the accompanying "external" configuration.
    ///
    /// For internal triggers, use other methods.
    #[inline]
    #[must_use]
    pub const fn from_external(signal: u8) -> Self {
        Self {
            select: TimerTriggerSelect(signal),
            source: TimerTriggerSource::External,
        }
    }
}

/// Timer decrement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
#[non_exhaustive]
pub enum TimerDecrement {
    /// Decrement on FlexIO clock, shift on timer output.
    FlexIoClockInputTimerOutput = 0b00,
    /// Decrement on trigger input (both edges), shift on timer output.
    TriggerInputTimerOutput = 0b01,
    /// Decrement on pin input (both edges), shift on pin input.
    PinInputPinInput = 0b10,
    /// Decrement on trigger input (both edges), shift on trigger input.
    TriggerInputTriggerInput = 0b11,
}

/// Timer reset conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
#[non_exhaustive]
pub enum TimerReset {
    /// Never reset.
    Never = 0b000,
    /// Pin equals timer output.
    PinEqualsOutput = 0b010,
    /// Trigger equals timer output.
    TriggerEqualsOutput = 0b011,
    /// Pin on rising edge.
    PinRisingEdge = 0b100,
    /// Trigger rising edge.
    TriggerRisingEdge = 0b110,
    /// Trigger either edge.
    TriggerEitherEdge = 0b111,
}

/// Timer disable condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TimerDisable {
    /// Never disabled.
    Never = 0b000,
    /// Disabled on previous timer disable.
    PreviousTimerDisable = 0b001,
    /// Disabled on timer compare.
    TimerCompare = 0b010,
    /// Disabled on timer compare and trigger low.
    TimerCompareAndTriggerLow = 0b011,
    /// Disabled on pin rising or falling edge.
    PinEitherEdge = 0b100,
    /// Disabled on pin rising or falling edge provided the trigger is high.
    PinEitherEdgeAndTriggerHigh = 0b101,
    /// Disabled on trigger falling edge.
    TriggerFallingEdge = 0b110,
}

/// Timer enable condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TimerEnable {
    /// Always enabled.
    Always = 0b000,
    /// Enabled on previous timer enable.
    PreviousTimerEnable = 0b001,
    /// Enabled on trigger high.
    TriggerHigh = 0b010,
    /// Enabled on trigger high and pin high.
    TriggerHighAndPinHigh = 0b011,
    /// Enabled on pin rising edge.
    PinRisingEdge = 0b100,
    /// Enabled on pin rising edge and trigger high.
    PinRisingEdgeAndTriggerHigh = 0b101,
    /// Enabled on trigger rising edge.
    TriggerRisingEdge = 0b110,
    /// Enabled on trigger rising or falling edge.
    TriggerEitherEdge = 0b111,
}

/// Timer stop bit configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TimerStopBit {
    /// Stop bit disabled.
    Disabled = 0b00,
    /// Stop bit enabled on timer compare.
    OnTimerCompare = 0b01,
    /// Stop bit enabled on timer disable.
    OnTimerDisable = 0b10,
    /// Stop bit enabled on timer compare and disable.
    OnTimerCompareAndDisable = 0b11,
}

/// Timer start bit configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TimerStartBit {
    /// Start bit disabled.
    Disabled = 0,
    /// Start bit enabled.
    Enabled = 1,
}

/// Timer output configuration.
///
/// Controls the initial state of the timer output when enabled and whether
/// timer reset events affect the output state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
pub enum TimerOutput {
    /// Timer output is logic one when enabled and is not affected by timer reset.
    OneNotAffectedByReset = 0b00,
    /// Timer output is logic zero when enabled and is not affected by timer reset.
    ZeroNotAffectedByReset = 0b01,
    /// Timer output is logic one when enabled and on timer reset.
    OneAffectedByReset = 0b10,
    /// Timer output is logic zero when enabled and on timer reset.
    ZeroAffectedByReset = 0b11,
}

/// Invalid pin selection error.
///
/// Either the pin identifier is invalid, or the pin is incompatible
/// with the FlexIO instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidPinError(());

/// A FlexIO input / output pin handle.
///
/// The handle is clone and copy. You're responsible for tracking
/// which FlexIO instance it's assigned to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Pin(u8);

impl Pin {
    /// Create a pin from a raw identifier.
    ///
    /// This call will not prepare any IOMUXC pad! If you're using this
    /// API, you're expected to do that yourself.
    pub const fn from_raw(pin: u8) -> Result<Self, InvalidPinError> {
        if pin < 32 {
            Ok(Self(pin))
        } else {
            Err(InvalidPinError(()))
        }
    }

    /// Construct a pin handle by consuming and preparing its pad object.
    ///
    /// This does not check for compatibility with a FlexIO instance!
    /// If you want that check, use [`FlexIo::try_prepare_pin`]. When
    /// this call returns, the provided pad is configured for the given
    /// FlexIO instance.
    pub fn from_iomuxc_pad<P: crate::iomuxc::flexio::Pin<N>, const N: u8>(mut pad: P) -> Self {
        crate::iomuxc::flexio::prepare(&mut pad);
        Self(P::OFFSET)
    }

    /// Returns the raw value of this pin.
    pub const fn pin(self) -> u32 {
        self.0 as u32
    }
}

/// Timer configuration.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimerConfig {
    /// The enabled timer mode.
    pub mode: TimerMode,
    /// Trigger trigger.
    pub trigger: TimerTrigger,
    /// Trigger polarity.
    pub trigger_polarity: TimerTriggerPolarity,
    /// Configurations for a pin, if enabled.
    pub pin: PinSelection,
    /// Timer output initial state and reset behavior.
    pub output: TimerOutput,
    /// Timer decrement mode.
    pub decrement: TimerDecrement,
    /// Timer reset condition.
    pub reset: TimerReset,
    /// Timer disable condition.
    pub disable: TimerDisable,
    /// Timer enable condition.
    pub enable: TimerEnable,
    /// Stop bit configuration.
    pub stop_bit: TimerStopBit,
    /// Start bit configuration.
    pub start_bit: TimerStartBit,
}

/// A FlexIO peripheral driver.
pub struct FlexIo {
    flexio: AnyInstance,
}

impl FlexIo {
    /// Create a new FlexIO driver from a RAL instance.
    ///
    /// When this returns, the FlexIO peripheral has been reset.
    /// It's enabled. You're responsible for configuring IOMUX pin settings.
    pub fn new<const N: u8>(flexio: ral::flexio::Instance<N>) -> Self {
        let mut this = Self {
            flexio: crate::into_any(flexio),
        };
        this.set_enable(false);
        this.reset();
        this.set_enable(true);
        this
    }

    /// Form a pin handle for this FlexIO block.
    ///
    /// Returns an error if the provided pad is incompatible with this
    /// FlexIO peripheral instance.
    pub fn try_prepare_pin<P: crate::iomuxc::flexio::Pin<N>, const N: u8>(
        &self,
        pad: P,
    ) -> Result<Pin, InvalidPinError> {
        if ral::flexio::number(&*self.flexio).unwrap() == N {
            Ok(Pin::from_iomuxc_pad(pad))
        } else {
            Err(InvalidPinError(()))
        }
    }

    /// Enable or disable the FlexIO peripheral.
    ///
    /// When disabled, the FlexIO clocks are stopped and shifter/timer
    /// state machines are held in reset.
    #[inline]
    pub fn set_enable(&mut self, enable: bool) {
        ral::modify_reg!(ral::flexio, self.flexio, CTRL, FLEXEN: enable as u32);
    }

    /// Returns `true` if FlexIO is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        ral::read_reg!(ral::flexio, self.flexio, CTRL, FLEXEN == 1)
    }

    /// Software reset the FlexIO peripheral.
    ///
    /// This resets all shifters and timers, clearing all status flags
    /// and FIFOs. The main control register is not affected.
    #[inline]
    pub fn reset(&mut self) {
        // SWRST doesn't self-clear. Clear it in software.
        ral::modify_reg!(ral::flexio, self.flexio, CTRL, SWRST: 1);
        ral::modify_reg!(ral::flexio, self.flexio, CTRL, SWRST: 0);
    }

    /// Enable (`true`) or disable (`false`) FlexIO operation in doze mode.
    #[inline]
    pub fn set_doze_enable(&mut self, enable: bool) {
        // DOZEN bit is inverted - 0 = enabled in doze, 1 = disabled in doze
        ral::modify_reg!(ral::flexio, self.flexio, CTRL, DOZEN: (!enable) as u32);
    }

    /// Enable (`true`) or disable (`false`) FlexIO operation in debug mode.
    #[inline]
    pub fn set_debug_enable(&mut self, enable: bool) {
        // But of course, DBGE bit isn't inverted...
        ral::modify_reg!(ral::flexio, self.flexio, CTRL, DBGE: enable as u32);
    }

    /// Read the shifter status flags.
    ///
    /// A set bit indicates that the shifter status flag is set:
    ///
    /// - For transmit shifters: buffer is empty and ready for new data.
    /// - For receive shifters: buffer is full and contains valid data.
    #[inline]
    pub fn shifter_status(&self) -> ShifterMask {
        let status = ral::read_reg!(ral::flexio, self.flexio, SHIFTSTAT);
        ShifterMask::from_bits_truncate(status)
    }

    /// Clear shifter status flags.
    ///
    /// Write 1 to clear the corresponding status flag.
    #[inline]
    pub fn clear_shifter_status(&self, flags: ShifterMask) {
        ral::write_reg!(ral::flexio, self.flexio, SHIFTSTAT, flags.bits());
    }

    /// Read the shifter error flags.
    ///
    /// A set bit indicates a shifter error:
    ///
    /// - For transmit shifters: buffer was written when not empty (overrun).
    /// - For receive shifters: buffer was not read before new data arrived (underrun).
    #[inline]
    pub fn shifter_error(&self) -> ShifterMask {
        let err = ral::read_reg!(ral::flexio, self.flexio, SHIFTERR);
        ShifterMask::from_bits_truncate(err)
    }

    /// Clear shifter error flags.
    ///
    /// Write 1 to clear the corresponding error flag.
    #[inline]
    pub fn clear_shifter_error(&self, flags: ShifterMask) {
        ral::write_reg!(ral::flexio, self.flexio, SHIFTERR, flags.bits());
    }

    /// Read the timer status flags.
    ///
    /// A set bit indicates that the timer status flag is set,
    /// meaning the timer has expired or reached a compare condition.
    #[inline]
    pub fn timer_status(&self) -> TimerMask {
        let status = ral::read_reg!(ral::flexio, self.flexio, TIMSTAT);
        TimerMask::from_bits_truncate(status)
    }

    /// Clear timer status flags.
    ///
    /// Write 1 to clear the corresponding status flag.
    #[inline]
    pub fn clear_timer_status(&self, flags: TimerMask) {
        ral::write_reg!(ral::flexio, self.flexio, TIMSTAT, flags.bits());
    }

    /// Set the shifter status interrupt enable flags.
    ///
    /// Enabled shifters will generate an interrupt when their status flag is set.
    /// This call performs a single write to the register. Make sure you're maintaining
    /// any previously enabled bits in your `flags` input.
    #[inline]
    pub fn set_shifter_status_interrupt_enable(&self, flags: ShifterMask) {
        ral::write_reg!(ral::flexio, self.flexio, SHIFTSIEN, flags.bits());
    }

    /// Read the shifter status interrupt enable flags.
    #[inline]
    pub fn shifter_status_interrupt_enable(&self) -> ShifterMask {
        let flags = ral::read_reg!(ral::flexio, self.flexio, SHIFTSIEN);
        ShifterMask::from_bits_truncate(flags)
    }

    /// Set the shifter error interrupt enable flags.
    ///
    /// Enabled shifters will generate an interrupt when their error flag is set.
    /// This call performs a single write to the register. Make sure you're maintaining
    /// any previously enabled bits in your `flags` input.
    #[inline]
    pub fn set_shifter_error_interrupt_enable(&self, flags: ShifterMask) {
        ral::write_reg!(ral::flexio, self.flexio, SHIFTEIEN, flags.bits());
    }

    /// Read the shifter error interrupt enable flags.
    #[inline]
    pub fn shifter_error_interrupt_enable(&self) -> ShifterMask {
        let flags = ral::read_reg!(ral::flexio, self.flexio, SHIFTEIEN);
        ShifterMask::from_bits_truncate(flags)
    }

    /// Set the timer interrupt enable flags.
    ///
    /// Enabled timers will generate an interrupt when their status flag is set.
    /// This call performs a single write to the register. Make sure you're maintaining
    /// any previously enabled bits in your `flags` input.
    #[inline]
    pub fn set_timer_interrupt_enable(&self, flags: TimerMask) {
        ral::write_reg!(ral::flexio, self.flexio, TIMIEN, flags.bits());
    }

    /// Read the timer interrupt enable flags.
    #[inline]
    pub fn timer_interrupt_enable(&self) -> TimerMask {
        let flags = ral::read_reg!(ral::flexio, self.flexio, TIMIEN);
        TimerMask::from_bits_truncate(flags)
    }

    /// Set the shifter status DMA enable flags.
    ///
    /// Enabled shifters will generate a DMA request when their status flag is set.
    /// This call performs a single write to the register. Make sure you're maintaining
    /// any previously enabled bits in your `flags` input.
    #[inline]
    pub fn set_shifter_status_dma_enable(&self, flags: ShifterMask) {
        ral::write_reg!(ral::flexio, self.flexio, SHIFTSDEN, flags.bits());
    }

    /// Read the shifter status DMA enable flags.
    #[inline]
    pub fn shifter_status_dma_enable(&self) -> ShifterMask {
        let flags = ral::read_reg!(ral::flexio, self.flexio, SHIFTSDEN);
        ShifterMask::from_bits_truncate(flags)
    }

    /// Configure and enable a shifter.
    pub fn set_shifter_config(&mut self, shifter: Shifter, config: &ShifterConfig) {
        let idx = shifter as usize;

        ral::write_reg!(ral::flexio, self.flexio, SHIFTCFG[idx],
            INSRC: config.input_source as u32,
            SSTOP: config.stop_bit as u32,
            SSTART: config.start_bit as u32,
            PWIDTH: config.parallel_width as u32
        );

        ral::write_reg!(ral::flexio, self.flexio, SHIFTCTL[idx],
            TIMSEL: config.timer as u32,
            TIMPOL: config.timer_polarity as u32,
            PINCFG: config.pin.config as u32,
            PINSEL: config.pin.select.0 as u32,
            PINPOL: config.pin.polarity as u32,
            SMOD: config.mode as u32
        );
    }

    /// Disable a shifter.
    #[inline]
    pub fn disable_shifter(&mut self, shifter: Shifter) {
        ral::modify_reg!(ral::flexio, self.flexio, SHIFTCTL[shifter as usize], SMOD: 0);
    }

    /// Write data to a shifter's buffer register.
    #[inline]
    pub fn write_shifter(&self, shifter: Shifter, data: u32) {
        ral::write_reg!(ral::flexio, self.flexio, SHIFTBUF[shifter as usize], data);
    }

    /// Write data to a shifter's buffer register with bit swapping.
    #[inline]
    pub fn write_shifter_bit_swapped(&self, shifter: Shifter, data: u32) {
        ral::write_reg!(
            ral::flexio,
            self.flexio,
            SHIFTBUFBIS[shifter as usize],
            data
        );
    }

    /// Read data from a shifter's buffer register.
    #[inline]
    pub fn read_shifter(&self, shifter: Shifter) -> u32 {
        ral::read_reg!(ral::flexio, self.flexio, SHIFTBUF[shifter as usize])
    }

    /// Read data from a shifter's buffer register with bit swapping.
    #[inline]
    pub fn read_shifter_bit_swapped(&self, shifter: Shifter) -> u32 {
        ral::read_reg!(ral::flexio, self.flexio, SHIFTBUFBIS[shifter as usize])
    }

    /// Configure and enable a timer.
    pub fn set_timer_config(&mut self, timer: Timer, config: &TimerConfig) {
        let idx = timer as usize;

        ral::write_reg!(ral::flexio, self.flexio, TIMCFG[idx],
            TIMOUT: config.output as u32,
            TIMDEC: config.decrement as u32,
            TIMRST: config.reset as u32,
            TIMDIS: config.disable as u32,
            TIMENA: config.enable as u32,
            TSTOP: config.stop_bit as u32,
            TSTART: config.start_bit as u32
        );

        ral::write_reg!(ral::flexio, self.flexio, TIMCTL[idx],
            TRGSEL: config.trigger.select.0 as u32,
            TRGPOL: config.trigger_polarity as u32,
            TRGSRC: config.trigger.source as u32,
            PINCFG: config.pin.config as u32,
            PINSEL: config.pin.select.0 as u32,
            PINPOL: config.pin.polarity as u32,
            TIMOD: config.mode as u32
        );
    }

    /// Disable a timer.
    #[inline]
    pub fn disable_timer(&mut self, timer: Timer) {
        ral::modify_reg!(ral::flexio, self.flexio, TIMCTL[timer as usize], TIMOD: 0);
    }

    /// Set the timer compare value.
    ///
    /// The interpretation of `compare` depends on the timer mode.
    #[inline]
    pub fn set_timer_compare(&self, timer: Timer, compare: u16) {
        ral::write_reg!(
            ral::flexio,
            self.flexio,
            TIMCMP[timer as usize],
            compare as u32
        );
    }

    /// Read the timer compare value.
    #[inline]
    pub fn timer_compare(&self, timer: Timer) -> u16 {
        ral::read_reg!(ral::flexio, self.flexio, TIMCMP[timer as usize]) as u16
    }

    /// Read the input data on each of the FlexIO pins, as a bitmask.
    #[inline]
    pub fn read_pins(&self) -> u32 {
        ral::read_reg!(ral::flexio, self.flexio, PIN)
    }
}
