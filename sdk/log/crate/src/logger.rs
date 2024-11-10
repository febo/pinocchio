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

/// An uninitialized byte.
const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::uninit();

/// Logger to efficiently format log messages.
///
/// The logger is a fixed size buffer that can be used to format log messages
/// before sending them to the log output. Any type that implements the `Log`
/// trait can be appended to the logger.
pub struct Logger<const BUFFER: usize> {
    // Byte buffer to store the log message.
    buffer: [MaybeUninit<u8>; BUFFER],

    // Remaining space in the buffer.
    offset: usize,
}

impl<const BUFFER: usize> Default for Logger<BUFFER> {
    fn default() -> Self {
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
    /// Append a value to the logger.
    #[inline(always)]
    pub fn append<T: Log>(&mut self, value: T) {
        if self.is_full() {
            if BUFFER > 0 {
                unsafe {
                    let last = self.buffer.get_unchecked_mut(BUFFER - 1);
                    last.write(TRUCATED);
                }
            }
        } else {
            self.offset += value.write(&mut self.buffer[self.offset..]);
        }
    }

    /// Log the message in the buffer.
    #[inline(always)]
    pub fn log(&self) {
        log_message(self);
    }

    /// Clear the buffer.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.offset = 0;
    }

    /// Check if the buffer is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.offset == 0
    }

    /// Check if the buffer is full.
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.offset == BUFFER
    }

    /// Get the length of the buffer.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.offset
    }

    /// Get the remaining space in the buffer.
    #[inline(always)]
    pub fn remaining(&self) -> usize {
        BUFFER - self.offset
    }
}

/// Log a message.
#[inline(always)]
pub fn log_message(message: &[u8]) {
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
    fn debug(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
        self.write(buffer)
    }

    fn write(&self, buffer: &mut [MaybeUninit<u8>]) -> usize;
}

/// Implement the log trait for unsigned integer types.
macro_rules! impl_log_for_unsigned_integer {
    ( $type:tt, $max_digits:literal ) => {
        impl Log for $type {
            fn write(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
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
                        let mut digits = [UNINIT_BYTE; $max_digits];
                        let mut offset = $max_digits;

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

                        // Number of available digits to write.
                        let available = $max_digits - offset;
                        // Size of the buffer.
                        let length = buffer.len();

                        // Determines if the value was truncated or not by calculating the
                        // number of digits that can be written.
                        let (overflow, written) = if available <= length {
                            (false, available)
                        } else {
                            (true, length)
                        };

                        unsafe {
                            let ptr = buffer.as_mut_ptr();
                            #[cfg(target_os = "solana")]
                            sol_memcpy_(
                                ptr as *mut _,
                                digits[offset..].as_ptr() as *const _,
                                length as u64,
                            );
                            #[cfg(not(target_os = "solana"))]
                            core::ptr::copy_nonoverlapping(digits[offset..].as_ptr(), ptr, written);
                        }

                        // There might not have been space for all the value.
                        if overflow {
                            unsafe {
                                let last = buffer.get_unchecked_mut(written - 1);
                                last.write(TRUCATED);
                            }
                        }

                        written
                    }
                }
            }
        }
    };
}

// Supported unsigned integer types.
impl_log_for_unsigned_integer!(u8, 3);
impl_log_for_unsigned_integer!(u16, 5);
impl_log_for_unsigned_integer!(u32, 10);
impl_log_for_unsigned_integer!(u64, 20);
impl_log_for_unsigned_integer!(u128, 39);

/// Implement the log trait for the signed integer types.
macro_rules! impl_log_for_signed {
    ( $type:tt, $max_digits:literal ) => {
        impl Log for $type {
            fn write(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
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
                        let mut delta = 0;

                        if *self < 0 {
                            unsafe {
                                buffer.get_unchecked_mut(0).write(b'-');
                            }
                            delta += 1;
                            value = -value
                        };

                        let mut digits = [UNINIT_BYTE; $max_digits];
                        let mut offset = $max_digits;

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

                        // Number of available digits to write.
                        let available = $max_digits - offset;
                        // Size of the buffer.
                        let length = buffer.len() - delta;

                        // Determines if the value was truncated or not by calculating the
                        // number of digits that can be written.
                        let (overflow, written) = if available <= length {
                            (false, available)
                        } else {
                            (true, length)
                        };

                        unsafe {
                            let ptr = buffer[delta..].as_mut_ptr();
                            #[cfg(target_os = "solana")]
                            sol_memcpy_(
                                ptr as *mut _,
                                digits[offset..].as_ptr() as *const _,
                                length as u64,
                            );
                            #[cfg(not(target_os = "solana"))]
                            core::ptr::copy_nonoverlapping(digits[offset..].as_ptr(), ptr, written);
                        }

                        // There might not have been space for all the value.
                        if overflow {
                            unsafe {
                                let last = buffer.get_unchecked_mut(written + delta - 1);
                                last.write(TRUCATED);
                            }
                        }

                        written + delta
                    }
                }
            }
        }
    };
}

// Supported signed integer types.
impl_log_for_signed!(i8, 3);
impl_log_for_signed!(i16, 5);
impl_log_for_signed!(i32, 10);
impl_log_for_signed!(i64, 19);
impl_log_for_signed!(i128, 39);

/// Implement the log trait for the &str type.
impl Log for &str {
    fn debug(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
        if buffer.is_empty() {
            return 0;
        }

        unsafe {
            buffer.get_unchecked_mut(0).write(b'"');
        }

        let mut offset = 1;
        offset += self.write(&mut buffer[offset..]);

        match buffer.len() - offset {
            0 => unsafe {
                buffer.get_unchecked_mut(offset - 1).write(TRUCATED);
            },
            _ => {
                unsafe {
                    buffer.get_unchecked_mut(offset).write(b'"');
                }
                offset += 1;
            }
        }

        offset
    }

    fn write(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
        let length = core::cmp::min(buffer.len(), self.len());
        let offset = &mut buffer[..length];

        for (d, s) in offset.iter_mut().zip(self.bytes()) {
            d.write(s);
        }

        // There might not have been space for all the value.
        if length != self.len() {
            unsafe {
                let last = buffer.get_unchecked_mut(length - 1);
                last.write(TRUCATED);
            }
        }

        length
    }
}

/// Implement the log trait for the slice type.
macro_rules! impl_log_for_slice {
    ( [$type:ident] ) => {
        impl<$type> Log for &[$type]
        where
            $type: Log
        {
            impl_log_for_slice!(@generate_write);
        }
    };
    ( [$type:ident; $size:ident] ) => {
        impl<$type, const $size: usize> Log for &[$type; $size]
        where
            $type: Log
        {
            impl_log_for_slice!(@generate_write);
        }
    };
    ( @generate_write ) => {
        fn write(&self, buffer: &mut [MaybeUninit<u8>]) -> usize {
            if buffer.is_empty() {
                return 0;
            }

            // Size of the buffer.
            let length = buffer.len();

            unsafe {
                buffer.get_unchecked_mut(0).write(b'[');
            }

            let mut offset = 1;

            for value in self.iter() {
                if offset >= length {
                    unsafe {
                        buffer.get_unchecked_mut(offset - 1).write(TRUCATED);
                    }
                    offset = length;
                    break;
                }

                if offset > 1 {
                    if offset + 2 >= length {
                        unsafe {
                            buffer.get_unchecked_mut(length - 1).write(TRUCATED);
                        }
                        offset = length;
                        break;
                    } else {
                        unsafe {
                            buffer.get_unchecked_mut(offset).write(b',');
                            buffer.get_unchecked_mut(offset + 1).write(b' ');
                        }
                        offset += 2;
                    }
                }

                offset += value.debug(&mut buffer[offset..]);
            }

            if offset < length {
                unsafe {
                    buffer.get_unchecked_mut(offset).write(b']');
                }
                offset += 1;
            }

            offset
        }
    };
}

// Supported slice types.
impl_log_for_slice!([T]);
impl_log_for_slice!([T; N]);
