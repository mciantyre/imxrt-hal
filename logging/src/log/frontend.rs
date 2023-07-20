//! Logging front-end.
//!
//! Bridges the `log` singleton with a `bbqueue` queue.
//! Asynchronous peripherals can then read the queue to
//! transport data.

use core::{
    cell::RefCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

use super::Filters;

use bbqueue as bbq;
use critical_section::Mutex;

struct Logger<'a, const N: usize> {
    producer: Mutex<RefCell<bbq::Producer<'a, N>>>,
    filters: Filters,
}

impl log::Log for Logger<'_, { crate::BUFFER_SIZE }> {
    fn enabled(&self, metadata: &::log::Metadata) -> bool {
        metadata.level() <= ::log::max_level() // The log level is appropriate
            && self.filters.is_enabled(metadata) // The target is in the filter list
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut writer = Writer(self);
            use core::fmt::Write;
            let _ = write!(
                writer,
                "[{} {}]: {}\r\n",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {
        // Not yet supported.
    }
}

struct Writer<'a, 'b, const N: usize>(&'b Logger<'a, N>);

impl<const N: usize> core::fmt::Write for Writer<'_, '_, N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        critical_section::with(|cs| {
            let prod = self.0.producer.borrow(cs);
            let mut prod = prod.borrow_mut();
            crate::try_write_producer(s.as_bytes(), &mut prod)
                .map_err(|_| core::fmt::Error)
                .map(|_| ())
        })
    }
}

static READY: AtomicBool = AtomicBool::new(false);
static mut LOGGER: MaybeUninit<Logger<'static, { crate::BUFFER_SIZE }>> = MaybeUninit::uninit();

/// Write bytes directly to the logging frontend.
///
/// This bypasses the `log` macros, allowing you to write whatever you want
/// directly into the logging queue. It returns the number of bytes written,
/// which may be zero.
///
/// If the number of bytes returned by this function is less than the size
/// of the buffer, it means that some data was not written. You could try
/// to write the remaining data, but the situation might not change if you
/// haven't polled the logger.
///
/// # Panics
///
/// Panics if the logger is not initialized.
pub fn write_raw(bytes: &[u8]) -> usize {
    // Safety: assert shows logger is initialized.
    let logger = unsafe {
        assert!(READY.load(Ordering::Acquire));
        LOGGER.assume_init_ref()
    };

    critical_section::with(|cs| {
        let prod = logger.producer.borrow(cs);
        let mut prod = prod.borrow_mut();
        crate::try_write_producer(bytes, &mut prod).unwrap_or(0)
    })
}

/// Initialize the logging frontend.
///
/// # Safety
///
/// Caller must ensure that this function is only called once.
pub(crate) unsafe fn init(
    producer: bbq::Producer<'static, { crate::BUFFER_SIZE }>,
    config: &super::LoggingConfig,
) -> Result<(), crate::AlreadySetError<()>> {
    assert!(!READY.load(Ordering::Acquire));
    // Safety: write to static mut. Assumed that this only happens once.
    // We should be preventing multiple callers with the critical section,
    // so the "only happens once" is to ensure that we're not changing the
    // static while the logger is active.
    LOGGER.write(Logger {
        producer: Mutex::new(RefCell::new(producer)),
        filters: super::Filters(config.filters),
    });
    READY.store(true, Ordering::Release);
    ::log::set_logger(LOGGER.assume_init_ref())
        .map(|_| ::log::set_max_level(config.max_level))
        .map_err(|_| crate::AlreadySetError::new(()))
}
