use core::{mem::MaybeUninit, ops::Deref, slice::from_raw_parts};

#[cfg(target_os = "solana")]
extern "C" {
    pub fn sol_log_(message: *const u8, len: u64);

    pub fn sol_memcpy_(dst: *mut u8, src: *const u8, n: u64);
}

#[cfg(not(target_os = "solana"))]
extern crate std;

/// Byte representation of the digits [0, 9].
const DIGITS: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];

/// Byte represeting a truncated log.
const TRUCATED: u8 = b'@';

/// Maximum number of digits for a U64.
const U64_DIGITS: usize = 20;

pub struct Logger<const BUFFER: usize> {
    // Byte buffer to store the log message.
    buffer: [MaybeUninit<u8>; BUFFER],

    // Remaining space in the buffer.
    offset: usize,
}

impl<const BUFFER: usize> Default for Logger<BUFFER> {
    fn default() -> Self {
        const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::uninit();
        Self {
            buffer: [UNINIT_BYTE; BUFFER],
            offset: 0,
        }
    }
}

impl<const BUFFER: usize> Deref for Logger<BUFFER> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { from_raw_parts(self.buffer.as_ptr() as *const _, self.offset) }
    }
}

impl<const BUFFER: usize> Logger<BUFFER> {
    #[inline(always)]
    pub fn append<T: Log>(&mut self, value: T) {
        if self.is_full() {
            if BUFFER > 0 {
                unsafe {
                    let last = self.buffer.get_unchecked_mut(BUFFER - 1);
                    last.assume_init_drop();
                    last.write(TRUCATED);
                }
            }
        } else {
            self.offset += value.log(&mut self.buffer[self.offset..]);
        }
    }

    #[inline(always)]
    pub fn log(&self) {
        log(self);
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.offset = 0;
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.offset == 0
    }

    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.offset == BUFFER
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.offset
    }

    #[inline(always)]
    pub fn remaining(&self) -> usize {
        BUFFER - self.offset
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
        if buffer.is_empty() {
            return 0;
        }

        match *self {
            // Handle zero as a special case.
            0 => {
                unsafe {
                    buffer.get_unchecked_mut(0).write(*DIGITS.get_unchecked(0));
                }
                1
            }
            mut value => {
                const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::uninit();
                let mut digits = [UNINIT_BYTE; U64_DIGITS];
                let mut offset = U64_DIGITS;

                while value > 0 {
                    let remainder = value % 10;
                    value /= 10;
                    offset -= 1;

                    unsafe {
                        digits
                            .get_unchecked_mut(offset)
                            .write(*DIGITS.get_unchecked(remainder as usize));
                    }
                }

                let length = core::cmp::min(buffer.len(), U64_DIGITS - offset);

                unsafe {
                    let ptr = buffer.as_mut_ptr();
                    #[cfg(target_os = "solana")]
                    sol_memcpy_(
                        ptr as *mut _,
                        digits[offset..].as_ptr() as *const _,
                        length as u64,
                    );
                    #[cfg(not(target_os = "solana"))]
                    core::ptr::copy_nonoverlapping(digits[offset..].as_ptr(), ptr, length);
                }

                // There might not have been space for all the value.
                if length != U64_DIGITS {
                    unsafe {
                        let last = buffer.get_unchecked_mut(length - 1);
                        last.write(TRUCATED);
                    }
                }

                length
            }
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
            unsafe {
                let last = buffer.get_unchecked_mut(length - 1);
                last.write(TRUCATED);
            }
        }

        length
    }
}
