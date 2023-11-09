//! Analog clock control module.
//!
//! Each module describes a system PLL. The symbols in this module depend on the
//! chip selection.

pub use crate::chip::config::ccm::analog::*;

/// The system PLL.
pub mod pll2 {
    /// PLL2 frequency (Hz).
    ///
    /// The reference manual notes that PLL2 should always run at 528MHz,
    /// so this constant assumes that PLL2's DIV_SELECT field isn't
    /// changed at runtime.
    pub const FREQUENCY: u32 = 528_000_000;

    /// The smallest PLL2_PFD divider.
    pub const MIN_FRAC: u8 = super::pll3::MIN_FRAC;
    /// The largest PLL2_PFD divider.
    pub const MAX_FRAC: u8 = super::pll3::MAX_FRAC;
}

/// The USB PLL.
///
/// When an implementation has multiple USB peripherals, this
/// PLL is associated with USB1.
pub mod pll3 {
    /// PLL3 frequency (Hz).
    ///
    /// The reference manual notes that PLL3 should always run at 480MHz,
    /// so this constant assumes that PLL3's DIV_SELECT field isn't
    /// changed at runtime.
    pub const FREQUENCY: u32 = 480_000_000;

    /// The smallest PLL3_PFD divider.
    pub const MIN_FRAC: u8 = 12;
    /// The largest PLL3_PFD divider.
    pub const MAX_FRAC: u8 = 35;

    use crate::ral;

    /// Restart the USB(1) PLL.
    pub fn restart(ccm_analog: &mut ral::ccm_analog::CCM_ANALOG) {
        loop {
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_USB1, ENABLE == 0) {
                ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_USB1_SET, ENABLE: 1);
                continue;
            }
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_USB1, POWER == 0) {
                ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_USB1_SET, POWER: 1);
                continue;
            }
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_USB1, LOCK == 0) {
                continue;
            }
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_USB1, BYPASS == 1) {
                ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_USB1_CLR, BYPASS: 1);
                continue;
            }
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_USB1, EN_USB_CLKS == 0) {
                ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_USB1_SET, EN_USB_CLKS: 1);
                continue;
            }
            break;
        }
    }
}
/// The Audio PLL
pub mod pll4 {

    /// The smallest PLL4_PFD divider.
    pub const MIN_FRAC: u8 = 12;
    /// The largest PLL4_PFD divider.
    pub const MAX_FRAC: u8 = 35;

    use crate::{common::ccm, ral};

    /// Restart the Audio PLL where all numerical options are enforced at build time.
    pub fn restart(
        ccm_analog: &mut ral::ccm_analog::CCM_ANALOG,
        div_select: u32,
        pll_num: u32,
        pll_denom: u32,
    ) {
        assert!(
            div_select >= 27 && div_select <= 54,
            "Audio PLL divider selection must be in range from 27 to 54 inclusive"
        );
        assert!(
            pll_num < pll_denom,
            "PLL requires numerator be less than the denominator"
        );
        let out_freq: u32 =
            ccm::XTAL_OSCILLATOR_HZ * div_select + (ccm::XTAL_OSCILLATOR_HZ * pll_num) / pll_denom;
        assert!(
            out_freq > 650_000_000 && out_freq <= 1_300_000_000,
            "Maximum PLL4 output range is from 650MHz to 1.3GHz"
        );
        // disable and power down pll
        loop {
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, BYPASS == 0) {
                ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_SET, BYPASS: 1);
                continue;
            }
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, ENABLE == 1) {
                ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_CLR, ENABLE: 1);
                continue;
            }
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, POWERDOWN == 0) {
                ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_SET, POWERDOWN: 1);
                continue;
            }
            break;
        }

        // set div_select, num, denom
        ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_NUM, pll_num);
        ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_DENOM, pll_denom);
        ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, DIV_SELECT: div_select);

        // power on, enable, and lock, but leave bypassed
        loop {
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, ENABLE == 0) {
                ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_SET, ENABLE: 1);
                continue;
            }
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, POWERDOWN == 1) {
                ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_CLR, POWERDOWN: 1);
                continue;
            }
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, LOCK == 0) {
                continue;
            }
            if ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, BYPASS == 1) {
                ral::write_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_CLR, BYPASS: 1);
                continue;
            }
            break;
        }
    }

    /// Get the current clock rate for the Audio PLL by inspecting the dividers
    pub fn clock_rate(ccm_analog: &ral::ccm_analog::CCM_ANALOG) -> u32 {
        let div_select: u32 = ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO, DIV_SELECT);
        let pll_num: u32 = ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_NUM);
        let pll_denom: u32 = ral::read_reg!(ral::ccm_analog, ccm_analog, PLL_AUDIO_DENOM);
        ccm::XTAL_OSCILLATOR_HZ * div_select + (ccm::XTAL_OSCILLATOR_HZ * pll_num) / pll_denom
    }
}
