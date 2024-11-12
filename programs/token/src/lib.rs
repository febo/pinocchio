#![no_std]

pub mod instructions;
pub mod state;

pinocchio_pubkey::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

use core::mem::MaybeUninit;

use pinocchio::pubkey::PUBKEY_BYTES;

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

pub struct IxData<'i> {
    pub bytes: &'i mut [MaybeUninit<u8>],
    pub current_size: usize,
    pub capacity: usize,
}

impl<'i> IxData<'i> {
    #[inline(always)]
    pub fn new(data: &'i mut [MaybeUninit<u8>]) -> Self {
        let capacity = data.len();
        Self {
            bytes: data,
            current_size: 0,
            capacity,
        }
    }

    #[inline(always)]
    pub fn write_bytes(&mut self, source: &[u8]) {
        if self.current_size + source.len() > self.capacity {
            return;
        }

        let start = self.current_size;
        let end = start + source.len();

        for (d, s) in &mut self.bytes[start..end].iter_mut().zip(source.iter()) {
            d.write(*s);
        }

        self.current_size += source.len();
    }

    #[inline(always)]
    pub fn write_optional_pubkey_bytes(&mut self, source: Option<&[u8; PUBKEY_BYTES]>) {
        if let Some(source) = source {
            self.write_bytes(&[1]);
            self.write_bytes(source);
        } else {
            self.write_bytes(&[0]);
        }
    }

    #[inline(always)]
    pub fn read_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.bytes.as_ptr() as *const u8, self.current_size) }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_write_bytes() {
        let mut data = [crate::UNINIT_BYTE; 50];
        let mut ix_data = crate::IxData::new(&mut data);
        assert_eq!(ix_data.current_size, 0);
        assert_eq!(ix_data.capacity, 50);
        ix_data.write_bytes(&[1, 2, 3, 4]);

        let optional_pubkey = Some(&[2; 32]);

        ix_data.write_optional_pubkey_bytes(optional_pubkey);

        assert!(ix_data.current_size == 37);

        ix_data.write_bytes(&[5, 6, 7, 8, 9, 10]);

        assert!(ix_data.current_size == 43);

        assert_eq!(
            ix_data.read_bytes(),
            &[
                1, 2, 3, 4, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2, 2, 2, 2, 2, 5, 6, 7, 8, 9, 10
            ]
        );
    }
}
