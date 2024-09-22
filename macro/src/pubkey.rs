pub use five8_const::decode_32_const;

#[inline(always)]
pub const fn decode(value: &str) -> [u8; 32] {
    decode_32_const(value)
}
