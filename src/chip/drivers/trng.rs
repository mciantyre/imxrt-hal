//! True random number generator.
//!
//! Provides basic support for the True Random Number Generator. The TRNG generates truly random
//! data and is intended for use as a generator of entropy.
//!
//! The TRNG is fairly slow - 15 minutes to an hour to generate 1 megabyte - so it probably
//! should only be used to generate a relatively small amount of entropy for a cryptographic
//! algorithm. Occasionally retrieving entropy from it won't necessarily need to block, as
//! this driver retrieves 512 bits at a time.
//!
//! # Example
//!
//! Enable the TRNG clock gate, wait to generate random data.
//!
//! ```no_run
//! use imxrt_hal as hal;
//! use imxrt_ral as ral;
//!
//! # || -> Option<()> {
//! let mut ccm = unsafe { ral::ccm::CCM::instance() };
//! hal::ccm::clock_gate::trng().set(&mut ccm, hal::ccm::clock_gate::ON);
//!
//! let mut trng = hal::trng::Trng::new(
//!     unsafe { ral::trng::TRNG::instance() },
//!     hal::trng::SampleMode::default(),
//!     hal::trng::RetryCount::default(),
//! );
//!
//! let random_data = nb::block!(trng.next_u32()).ok()?;
//! # Some(()) }();
//! ```

use core::fmt;

use crate::ral::trng;
use crate::ral::{modify_reg, read_reg, write_reg};

/// TRNG sampling mode
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SampleMode {
    /// von Neumann data in both entropy shifter and statistical checks. Approximately 4x slower
    /// than the other modes.
    VonNeumann = trng::MCTL::SAMP_MODE::RW::SAMP_MODE_0,
    /// Raw data in both entropy shifter and statistical checks. Likely lower quality than the
    /// other two modes.
    Raw = trng::MCTL::SAMP_MODE::RW::SAMP_MODE_1,
    /// von Neumann data in entropy shifter, raw data in statistical checks
    VonNeumannRaw = trng::MCTL::SAMP_MODE::RW::SAMP_MODE_2,
}

impl Default for SampleMode {
    /// Returns `VonNeumannRaw`.
    fn default() -> Self {
        // "Set sample mode of the TRNG ring oscillator to Von Neumann, for better random data.
        // It is optional." <- SDK, explaining why they set sample mode to 0 (VN)
        // Teensyduino uses VNRaw, the reason appears to be this post:
        // https://forum.pjrc.com/threads/54711-Teensy-4-0-First-Beta-Test?p=195000&viewfull=1#post195000
        // VNRaw appears to produce equivalent quality output and is ~4x faster than VN.
        // As such, VonNeumannRaw is the default SampleMode here.
        Self::VonNeumannRaw
    }
}

/// The true random number generator.
pub struct Trng {
    reg: trng::TRNG,
    block: [u32; 16],
    index: usize,
}

impl fmt::Debug for Trng {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TRNG")
            .field("block", &self.block)
            .field("index", &self.index)
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Trng {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Trng {{ block: {}, index: {} }}", self.block, self.index)
    }
}

/// The number of retry attempts.
///
/// Describes the number of times to retry
/// after a test failure before an error is declared. Valid
/// range `1..=15`. The default retry count is the largest
/// possible value.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryCount(u32);

impl RetryCount {
    /// The default reset count.
    pub const DEFAULT: u32 = 15;

    /// Create a new retry count.
    ///
    /// Returns `None` if `retry_count` is not in the half-closed range
    /// `1..=15`.
    pub fn new(retry_count: u32) -> Option<Self> {
        (1..=15)
            .contains(&retry_count)
            .then_some(RetryCount(retry_count))
    }
}

impl Default for RetryCount {
    fn default() -> Self {
        RetryCount(Self::DEFAULT)
    }
}

impl Trng {
    /// Create a TRNG given a sampling mode and retry count.
    ///
    /// To select the default sampling mode and retry counts, use `Default::default()`.
    pub fn new(reg: trng::TRNG, sample_mode: SampleMode, retry_count: RetryCount) -> Self {
        // Not supported: locking TRNG to prevent programmability.
        // A number of things here are likely configurable if you have access to the SRM.
        // Without it only limited configuration is safe. Garbage configs lead to endless errors.

        modify_reg!(trng, reg, MCTL, PRGM: 1);
        modify_reg!(trng, reg, MCTL, RST_DEF: 1);
        // enter program mode and reset to defaults
        // it isn't clear what defaults it actually sets, so let's set the tests manually

        // All these values are sourced only from the #defines in the MCUXpresso TRNG driver for the
        // IMXRT1062. The doc comments contain both typos and completely wrong values.

        write_reg!(trng, reg, SCMISC, RTY_CT: retry_count.0, LRUN_MAX: 34); // _RUN_MAX_LIMIT

        // Note: The SDK uses _MAX and _MIN for values, but the registers use the max value and a
        // range. The SDK _MIN values are expressed as (max - range), making it easy to ensure
        // that these values are correct.
        write_reg!(trng, reg, SCML, MONO_MAX: 1384, MONO_RNG: 268); // _MONOBIT_
        write_reg!(trng, reg, SCR1L, RUN1_MAX: 405, RUN1_RNG: 178); // _RUNBIT1_
        write_reg!(trng, reg, SCR2L, RUN2_MAX: 220, RUN2_RNG: 122); // _RUNBIT2_
        write_reg!(trng, reg, SCR3L, RUN3_MAX: 125, RUN3_RNG: 88); // _RUNBIT3_
        write_reg!(trng, reg, SCR4L, RUN4_MAX: 75, RUN4_RNG: 64); // _RUNBIT4_
        write_reg!(trng, reg, SCR5L, RUN5_MAX: 47, RUN5_RNG: 46); // _RUNBIT5_
        write_reg!(trng, reg, SCR6PL, RUN6P_MAX: 47, RUN6P_RNG: 46); // _RUNBIT6PLUS_

        write_reg!(trng, reg, PKRMAX, PKR_MAX: 26912); // _POKER_MAXIMUM
        write_reg!(trng, reg, PKRRNG, PKR_RNG: 2467);

        write_reg!(trng, reg, FRQMAX, FRQ_MAX: 25600); // _FREQUENCY_MAXIMUM
        write_reg!(trng, reg, FRQMIN, FRQ_MIN: 1600); // _FREQUENCY_MINIMUM

        write_reg!(trng, reg, SDCTL, SAMP_SIZE: 2500, ENT_DLY: 3200); // _SAMPLE_SIZE, _ENTROPY_DELAY
        write_reg!(trng, reg, SBLIM, SB_LIM: 63); // _SPARSE_BIT_LIMIT

        // set sample mode, exit program mode
        modify_reg!(trng, reg, MCTL, SAMP_MODE: sample_mode as u32);
        modify_reg!(trng, reg, MCTL, PRGM: 0);
        // for 1015, 1021, maybe other non i.MX chips: set TRNG_ACC to 1 here
        read_reg!(trng, reg, ENT[15]);
        // reading ENT15 triggers new entropy generation

        Self {
            reg,
            block: [0; 16],
            index: 16, // equal to len, to trigger immediate retrieval
        }
    }

    /// Return the next randomly-generated `u32`. May need to retrieve another block of random numbers.
    ///
    /// Returns "would block" if we're not ready to read entropy; try again. See the module-level
    /// example for how to block.
    pub fn next_u32(&mut self) -> nb::Result<u32, Error> {
        self.retrieve_if_needed()?;
        let data = nb::Result::Ok(self.block[self.index]);
        self.index += 1;
        data
    }

    /// Retrieve another block of random numbers if we've used them all up.
    fn retrieve_if_needed(&mut self) -> nb::Result<(), Error> {
        if self.index >= self.block.len() {
            self.retrieve()?;
            self.index = 0;
        }
        Ok(())
    }

    /// Retrieve another block of random numbers.
    fn retrieve(&mut self) -> nb::Result<(), Error> {
        let mctl = read_reg!(trng, self.reg, MCTL);
        if (mctl & trng::MCTL::ERR::mask) != 0 {
            let flags = self.get_error_flags();
            write_reg!(trng, self.reg, MCTL, mctl); // write reg back to clear error
            return Err(nb::Error::Other(Error(flags)));
        }
        if (mctl & trng::MCTL::ENT_VAL::mask) == 0 {
            return Err(nb::Error::WouldBlock); // not ready to read entropy
        }
        for idx in 0..self.reg.ENT.len() {
            self.block[idx] = read_reg!(trng, self.reg, ENT[idx]);
        }

        read_reg!(trng, self.reg, ENT[0]);
        // SDK (fsl_trng.c):
        //     Dummy read. Defect workaround.
        //     TRNG could not clear ENT_VAL flag automatically, application
        //     had to do a dummy reading operation for anyone TRNG register
        //     to clear it firstly, then to read the RTENT0 to RTENT15 again
        // This appears unnecessary on the 1062? done anyway in case it's necessary for another chip
        Ok(())
    }

    /// Retrieve all known error flags.
    fn get_error_flags(&self) -> ErrorFlags {
        let status = read_reg!(trng, self.reg, STATUS) & 0xFFFF;
        // all the error flags in STATUS are in the low 16 bits
        let mut flags = ErrorFlags::from_bits_truncate(status);
        flags.set(
            ErrorFlags::FCT_FAIL,
            read_reg!(trng, self.reg, MCTL, FCT_FAIL) == 1,
        );
        flags
    }

    /// Release the TRNG in a disabled state.
    ///
    /// This preserves any previously set retry count, sample mode,
    /// and peripheral settings. However, the register block is returned
    /// in a disabled state.
    pub fn release_disabled(self) -> trng::TRNG {
        modify_reg!(trng, self.reg, MCTL, PRGM: 1);
        while read_reg!(trng, self.reg, MCTL, TSTOP_OK) == 0 {
            core::hint::spin_loop();
        }
        self.reg
    }
}

/// A TRNG error occurred, such as a statistical test failing.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Error(pub ErrorFlags);

bitflags::bitflags! {
    /// Specific errors that may occur during entropy generation
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ErrorFlags : u32 {
        // STATUS register starts here (automatically set from bits)
        /// 1-bit run sampling 0s test failed
        const TF1BR0 = 1 << 0;
        /// 1-bit run sampling 1s test failed
        const TF1BR1 = 1 << 1;
        /// 2-bit run sampling 0s test failed
        const TF2BR0 = 1 << 2;
        /// 2-bit run sampling 1s test failed
        const TF2BR1 = 1 << 3;
        /// 3-bit run sampling 0s test failed
        const TF3BR0 = 1 << 4;
        /// 3-bit run sampling 1s test failed
        const TF3BR1 = 1 << 5;
        /// 4-bit run sampling 0s test failed
        const TF4BR0 = 1 << 6;
        /// 4-bit run sampling 1s test failed
        const TF4BR1 = 1 << 7;
        /// 5-bit run sampling 0s test failed
        const TF5BR0 = 1 << 8;
        /// 5-bit run sampling 1s test failed
        const TF5BR1 = 1 << 9;
        /// 6-plus-bit run sampling 0s test failed
        const TF6PBR0 = 1 << 10;
        /// 6-plus-bit run sampling 1s test failed
        const TF6PBR1 = 1 << 11;
        /// Sparse bit test failed
        const TFSB = 1 << 12;
        /// Long run test failed
        const TFLR = 1 << 13;
        /// Poker test failed
        const TFP = 1 << 14;
        /// Mono bit test failed
        const TFMB = 1 << 15;
        // MCTL register starts here (set manually)
        /// Count taken during entropy generation was outside the defined range of FRQ_MIN to FRQ_MAX
        const FCT_FAIL = 1 << 16;
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ErrorFlags {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "ErrorFlags({=u32:#x})", self.bits());
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An error occurred in the TRNG module")
    }
}

#[cfg(test)]
mod tests {
    use super::RetryCount;

    #[test]
    fn retry_count() {
        assert!(RetryCount::new(0).is_none());
        for count in 1..16 {
            assert!(RetryCount::new(count).is_some());
        }
        assert!(RetryCount::new(16).is_none());
    }
}
