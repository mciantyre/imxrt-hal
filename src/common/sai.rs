//! Synchronous Audio Interface.
//!
//! [`Sai`] provides a pair of synchronous audio word streams containaing stereo data.
//!
//! This driver also exposes the peripheral's lower-level, hardware-dependent audio stream
//! configuration and FIFO pair.
//!
//! Each Sai instance has at minimum a tx and rx data line. Each data line supports up to 32 audio
//! words per frame. Audio words are 8 to 32 bits. Frames can be used to send multichannel audio
//! data over a single serial stream such as stereo audio.
//!
//! Each data line comes with its own 32x32 FIFO allowing for a full frame to be sent and/or received
//! without software interaction.
//!
//! The configuration of the SAI is encoded in configuration structure that can be used with a singular
//! configure method.

use crate::ccm;
use crate::iomuxc::{consts, sai};
use crate::ral;

pub enum SaiByteOrder {
    LSB = 0,
    MSB = 1,
}

pub enum SaiClockPolarity {
    ActiveHigh = 0,
    ActiveLow = 1,
}

impl SaiClockPolarity {
    const SampleOnRising: SaiClockPolarity = SaiClockPolarity::ActiveLow;
    const SampleOnFalling: SaiClockPolarity = SaiClockPolarity::ActiveHigh;
}

pub struct SaiFrameSync {
    sync_width: u8,
    sync_early: bool,
    polarity: SaiClockPolarity,
}

pub enum SaiMclkSource {
    Sysclk = 0,
    Select1 = 1,
    Select2 = 2,
    Select3 = 3,
}

pub enum SaiBclkSource {
    Bus = 0,
    Opt1 = 1,
    Opt2 = 2,
    Opt3 = 3,
}

impl SaiBclkSource {
    const MclkDiv: SaiBclkSource = SaiBclkSource::Opt1;
}

pub struct SaiBitClock {
    src_swap: bool,
    input_delay: bool,
    polarity: SaiClockPolarity,
    source: SaiBclkSource,
}

pub struct SaiSerialData {
    byte_order: SaiByteOrder,
    word_length: u8,
    frame_length: u8,
}

pub enum SaiMasterSlave {
    Master = 0,
    Slave = 1,
    BclkMasterFrameSyncSlave = 2,
    BclkSlaveFrameSyncMaster = 3,
}

pub enum SaiFrameSyncMode {
    Async = 0,
    Sync = 1,
    SyncWithOtherTx = 2,
    SyncwithOtherRx = 3,
}

pub struct SaiConfig {
    serial_data: SaiSerialData,
    frame_sync: SaiFrameSync,
    bit_clock: SaiBitClock,
    master_slave: SaiMasterSlave,
    sync_mode: SaiFrameSyncMode,
    start_channel: u8,
    end_channel: u8,
    channel_mask: u8,
    channels: u8,
}

impl SaiConfig {
    fn i2s(bit_width: u8, channel_mask: u8) -> Self {
        SaiConfig {
            serial_data: SaiSerialData {
                byte_order: SaiByteOrder::MSB,
                word_length: bit_width,
                frame_length: 2,
            },
            frame_sync: SaiFrameSync {
                sync_width: bit_width,
                sync_early: true,
                polarity: SaiClockPolarity::ActiveLow,
            },
            bit_clock: SaiBitClock {
                src_swap: false,
                input_delay: false,
                polarity: SaiClockPolarity::SampleOnRising,
                source: SaiBclkSource::MclkDiv,
            },
            master_slave: SaiMasterSlave::Master,
            sync_mode: SaiFrameSyncMode::Async,
            start_channel: 0,
            end_channel: 0,
            channel_mask,
            channels: 0,
        }
    }
}

pub struct TxPins<TxSync, TxBclk> {
    /// Frame sync pin
    pub sync: TxSync,
    /// Bit clock pin
    pub bclk: TxBclk,
}

pub struct RxPins<RxSync, RxBclk> {
    /// Frame sync pin
    pub sync: RxSync,
    /// Bit clock pin
    pub bclk: RxBclk,
}

/// A SAI peripheral instance
pub struct Sai<const N: u8> {
    pub(super) sai: ral::sai::Instance<N>,
}

// An instance of a SAI transmitter
pub struct SaiTx<P, const N: u8> {
    pub(super) sai: ral::sai::Instance<N>,
    pins: P,
}

/// An instance of a SAI transmitter channel
///
/// NOTE: A SAI channel is representative of the physical data pin and associated FIFO
/// not a time division channel for packing multi-channel audio in each frame
pub struct SaiTxChannel<P, const N: u8, const C: u8> {
    sai: ral::sai::Instance<N>,
    tx_data: P,
}

/// An instance of a SAI receiver
pub struct SaiRx<P, const N: u8> {
    pub(super) sai: ral::sai::Instance<N>,
    pins: P,
}

/// An instance of a SAI receiver channel
///
/// NOTE: A SAI channel is representative of the physical data pin and associated FIFO
/// not a time division channel for packing multi-channel audio in each frame.
pub struct SaiRxChannel<P, const N: u8, const C: u8> {
    sai: ral::sai::Instance<N>,
    rx_data: P,
}

/// Trait a SaiRx implements for each channel available for taking
pub trait TakeRxChannel<P: sai::RxDataSignal, const N: u8, const C: u8> {
    fn take_channel(&mut self, rx_data: P) -> Result<SaiRxChannel<P, N, C>, SaiError>;
}

// Trait a SaiTx implements for each channel available for taking
pub trait TakeTxChannel<P: sai::TxDataSignal, const N: u8, const C: u8> {
    fn take_channel(&mut self, tx_data: P) -> Result<SaiTxChannel<P, N, C>, SaiError>;
}

/// Possible errors when interfacing the SAI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaiError {
    /// The transaction frame size is incorrect.
    ///
    /// The frame size, in bits, must be between 8 bits and
    /// 4096 bits.
    FrameSize,
    /// Caller provided no data.
    NoData,
    /// Channel already taken
    ChannelTaken,
}

fn reset_tx(regs: &ral::sai::RegisterBlock) {
    ral::write_reg!(ral::sai, regs, TCSR, SR: 1, FR: 1);
    ral::modify_reg!(ral::sai, regs, TCSR, SR: 0);
    ral::write_reg!(ral::sai, regs, TCR2, 0);
    ral::write_reg!(ral::sai, regs, TCR3, 0);
    ral::write_reg!(ral::sai, regs, TCR4, 0);
    ral::write_reg!(ral::sai, regs, TCR5, 0);
    ral::write_reg!(ral::sai, regs, TMR, 0);
}

fn reset_rx(regs: &ral::sai::RegisterBlock) {
    ral::write_reg!(ral::sai, regs, RCSR, SR: 1, FR: 1);
    ral::modify_reg!(ral::sai, regs, RCSR, SR: 0);
    ral::write_reg!(ral::sai, regs, RCR2, 0);
    ral::write_reg!(ral::sai, regs, RCR3, 0);
    ral::write_reg!(ral::sai, regs, RCR4, 0);
    ral::write_reg!(ral::sai, regs, RCR5, 0);
    ral::write_reg!(ral::sai, regs, RMR, 0);
}

impl<const N: u8> Sai<N> {
    /// The peripheral instance.
    pub const N: u8 = N;

    /// Initialize the SAI instance by resetting everything
    pub fn init(mut sai: ral::sai::Instance<N>, sample_rate: u32, cfg: &SaiConfig) -> Self {
        reset_tx(&mut sai);
        reset_rx(&mut sai);
        Sai { sai }
    }

    /// Take the SAI transmit handle given a set of TxPins
    pub fn take_tx<TxSync, TxBclk, P>(&mut self, pins: P) -> Result<SaiTx<P, N>, SaiError> {
        //TODO check if Tx already taken
        Ok(SaiTx {
            sai: unsafe { ral::sai::Instance::new(&*self.sai) },
            pins,
        })
    }

    /// Take the SAI receive handle given a set of RxPins
    pub fn take_rx<RxSync: sai::Signal, RxBclk: sai::Signal, P>(
        &mut self,
        pins: P,
    ) -> Result<SaiRx<P, N>, SaiError> {
        //TODO check if Rx already taken
        Ok(SaiRx {
            sai: unsafe { ral::sai::Instance::new(&*self.sai) },
            pins,
        })
    }
}

//TODO automate the Take[Tx/Rx]Channel impls with a macro across the various SAI instances available on the part
impl<P> TakeTxChannel<P, 1, 1> for Sai<1>
where
    P: sai::TxDataSignal,
{
    fn take_channel(&mut self, tx_data: P) -> Result<SaiTxChannel<P, 1, 1>, SaiError> {
        //TODO check channel mask and update it if needed
        Ok(SaiTxChannel {
            sai: unsafe { ral::sai::Instance::new(&*self.sai) },
            tx_data,
        })
    }
}

//TODO automate the Take[Tx/Rx]Channel impls with a macro across the various SAI instances available on the part
impl<P> TakeRxChannel<P, 1, 1> for Sai<1>
where
    P: sai::RxDataSignal,
{
    fn take_channel(&mut self, rx_data: P) -> Result<SaiRxChannel<P, 1, 1>, SaiError> {
        //TODO check channel mask and update it if needed
        Ok(SaiRxChannel {
            sai: unsafe { ral::sai::Instance::new(&*self.sai) },
            rx_data,
        })
    }
}

/// Trait to write a single machine word of audio data, potentially packed, to a channel
trait AudioWriteWord {
    fn write_word(&mut self, word: u32);
}

/// Trait to read a single machine word of audio data, potentially packed, from a channel
trait AudioReadWord {
    fn read_word(&mut self) -> u32;
}

impl<P, const N: u8, const C: u8> AudioWriteWord for SaiTxChannel<P, N, C> {
    #[inline]
    fn write_word(&mut self, word: u32) {
        ral::write_reg!(ral::sai, self.sai, TDR[C as usize], word);
    }
}

impl<P, const N: u8, const C: u8> AudioReadWord for SaiRxChannel<P, N, C> {
    #[inline]
    fn read_word(&mut self) -> u32 {
        ral::read_reg!(ral::sai, self.sai, RDR[C as usize])
    }
}

/// Trait for writing a full frame of unpacked audio data
trait AudioWriteFrame<T, const L: usize> {
    fn write_frame(&mut self, buf: &[T; L]);
}

/// Trait for reading a full frame of unpacked audio data
trait AudioReadFrame<T, const L: usize> {
    fn read_frame(&mut self, buf: &mut [T; L]);
}
