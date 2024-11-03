use core::{mem::MaybeUninit, ops::Deref, slice::from_raw_parts};

#[cfg(target_os = "solana")]
extern "C" {
    pub fn sol_log_(message: *const u8, len: u64);
}

/// Byte representation of the digits [0, 9].
const DIGITS: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];

/// Byte represeting a truncated log.
const TRUCATED: u8 = b'@';

pub struct Logger<const BUFFER: usize> {
    // Byte buffer to store the log message.
    buffer: [MaybeUninit<u8>; BUFFER],

    // Remaining space in the buffer.
    remaining: usize,
}

impl<const BUFFER: usize> Default for Logger<BUFFER> {
    fn default() -> Self {
        const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::uninit();
        Self {
            buffer: [UNINIT_BYTE; BUFFER],
            remaining: BUFFER,
        }
    }
}

impl<const BUFFER: usize> Deref for Logger<BUFFER> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { from_raw_parts(self.buffer.as_ptr() as *const _, BUFFER - self.remaining) }
    }
}

impl<const BUFFER: usize> Logger<BUFFER> {
    #[inline(always)]
    pub fn append<T: Log>(&mut self, value: T) {
        if self.remaining == 0 {
            if BUFFER > 0 {
                let last = BUFFER - 1;
                unsafe {
                    self.buffer[last].assume_init_drop();
                    self.buffer[last].write(TRUCATED);
                }
            }
        } else {
            let size = value.log(&mut self.buffer[BUFFER - self.remaining..]);
            self.remaining = self.remaining.saturating_sub(size);
        }
    }

    #[inline(always)]
    pub fn log(&self) {
        log(self);
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.remaining = BUFFER;
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.remaining == BUFFER
    }

    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.remaining == 0
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        BUFFER - self.remaining
    }
}

#[inline(always)]
pub fn log(message: &[u8]) {
    #[cfg(target_os = "solana")]
    unsafe {
        sol_log_(message.as_ptr(), message.len() as u64);
    }
    #[cfg(not(target_os = "solana"))]
    {
        extern crate std;

        let message = core::str::from_utf8(message).unwrap();
        std::println!("{}", message);
    }
}

/// Trait to specify the log behavior for a type.
pub trait Log {
    fn log(&self, buffer: &mut [MaybeUninit<u8>]) -> usize;
}

impl Log for u64 {
    fn log(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
        let mut value = *self;
        let mut offset = 0;

        // Handle zero as a special case.
        if value == 0 {
            buffer[offset].write(DIGITS[0]);
            1
        } else {
            while value > 0 && offset < buffer.len() {
                let remainder = value % 10;
                buffer[offset].write(DIGITS[remainder as usize]);
                offset += 1;
                value /= 10;
            }
            // Reverse the slice to get the correct order.
            buffer[0..offset].reverse();

            // There maight not have been space for all the value.
            if value > 0 {
                let last = offset - 1;
                unsafe {
                    // Drop the previous value.
                    buffer[last].assume_init_drop();
                    buffer[last].write(TRUCATED);
                }
            }

            offset
        }
    }
}

impl Log for &str {
    fn log(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
        let length = core::cmp::min(buffer.len(), self.len());
        let offset = &mut buffer[..length];

        for (d, s) in offset.iter_mut().zip(self.bytes()) {
            d.write(s);
        }

        // There maight not have been space for all the value.
        if length != self.len() {
            let last = length - 1;
            unsafe {
                // Drop the previous value.
                buffer[last].assume_init_drop();
                buffer[last].write(TRUCATED);
            }
        }

        length
    }
}
