//! This account contains the current cluster rent.
//!
//! This is required for the rent sysvar implementation.

use super::Sysvar;
use crate::impl_sysvar_get;

/// Default rental rate in lamports/byte-year.
///
/// This calculation is based on:
/// - 10^9 lamports per SOL
/// - $1 per SOL
/// - $0.01 per megabyte day
/// - $3.65 per megabyte year
pub const DEFAULT_LAMPORTS_PER_BYTE_YEAR: u64 = 1_000_000_000 / 100 * 365 / (1024 * 1024);

/// Default amount of time (in years) the balance has to include rent for the
/// account to be rent exempt.
pub const DEFAULT_EXEMPTION_THRESHOLD: f64 = 2.0;

/// Default percentage of collected rent that is burned.
///
/// Valid values are in the range [0, 100]. The remaining percentage is
/// distributed to validators.
pub const DEFAULT_BURN_PERCENT: u8 = 50;

/// Account storage overhead for calculation of base rent.
///
/// This is the number of bytes required to store an account with no data. It is
/// added to an accounts data length when calculating [`Rent::minimum_balance`].
pub const ACCOUNT_STORAGE_OVERHEAD: u64 = 128;

const EXEMPTION_THRESHOLD_SCALE_FACTOR: u64 = 1_000_000_000;

/// Rent sysvar data
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct Rent {
    /// Rental rate in lamports per byte-year
    pub lamports_per_byte_year: u64,

    /// Exemption threshold in years
    pub exemption_threshold: f64,

    /// Burn percentage
    pub burn_percent: u8,
}

/// Calculates the rent for a given number of bytes and duration
///
/// # Arguments
///
/// * `bytes` - The number of bytes to calculate rent for
/// * `years` - The number of years to calculate rent for
///
/// # Returns
///
/// The total rent in lamports
impl Rent {
    /// Calculate how much rent to burn from the collected rent.
    ///
    /// The first value returned is the amount burned. The second is the amount
    /// to distribute to validators.
    pub fn calculate_burn(&self, rent_collected: u64) -> (u64, u64) {
        let burned_portion = (rent_collected * u64::from(self.burn_percent)) / 100;
        (burned_portion, rent_collected - burned_portion)
    }

    /// Rent due on account's data length with balance.
    pub fn due(&self, balance: u64, data_len: usize, years_elapsed: f64) -> RentDue {
        if self.is_exempt(balance, data_len) {
            RentDue::Exempt
        } else {
            RentDue::Paying(self.due_amount(data_len, years_elapsed))
        }
    }

    /// Rent due for account that is known to be not exempt.
    pub fn due_amount(&self, data_len: usize, years_elapsed: f64) -> u64 {
        let actual_data_len = data_len as u64 + ACCOUNT_STORAGE_OVERHEAD;
        let lamports_per_year = self.lamports_per_byte_year * actual_data_len;
        (lamports_per_year as f64 * years_elapsed) as u64
    }

    /// Calculates the minimum balance for rent exemption.
    ///
    /// # Arguments
    ///
    /// * `data_len` - The number of bytes in the account
    ///
    /// # Returns
    ///
    /// The minimum balance in lamports for rent exemption.
    pub fn minimum_balance(&self, data_len: usize) -> u64 {
        let bytes = data_len as u64;
        (((ACCOUNT_STORAGE_OVERHEAD + bytes) * self.lamports_per_byte_year) as f64
            * self.exemption_threshold) as u64
    }

    /// Determines if an account can be considered rent exempt.
    ///
    /// # Arguments
    ///
    /// * `lamports` - The balance of the account in lamports
    /// * `data_len` - The size of the account in bytes
    ///
    /// # Returns
    ///
    /// `true`` if the account is rent exempt, `false`` otherwise.
    pub fn is_exempt(&self, lamports: u64, data_len: usize) -> bool {
        lamports >= self.minimum_balance(data_len)
    }

    /// Calculates the minimum balance for rent exemption.
    ///
    /// # Arguments
    ///
    /// * `data_len` - The number of bytes in the account
    ///
    /// # Returns
    ///
    /// The minimum balance in lamports for rent exemption.
    #[inline]
    pub fn minimum_balance_scaled(&self, data_len: usize) -> u64 {
        let bytes = data_len as u64;

        (((ACCOUNT_STORAGE_OVERHEAD + bytes) * self.lamports_per_byte_year)
            * self.exemption_threshold_scaled())
            / EXEMPTION_THRESHOLD_SCALE_FACTOR
    }

    /// Determines if an account can be considered rent exempt.
    ///
    /// # Arguments
    ///
    /// * `lamports` - The balance of the account in lamports
    /// * `data_len` - The size of the account in bytes
    ///
    /// # Returns
    ///
    /// `true`` if the account is rent exempt, `false`` otherwise.
    #[inline]
    pub fn is_exempt_scaled(&self, lamports: u64, data_len: usize) -> bool {
        lamports.saturating_mul(EXEMPTION_THRESHOLD_SCALE_FACTOR)
            >= self.minimum_balance_scaled(data_len)
    }

    /// Returns the exemption threshold scaled by `EXEMPTION_THRESHOLD_SCALE_FACTOR`.
    fn exemption_threshold_scaled(&self) -> u64 {
        let bits = self.exemption_threshold.to_bits();
        // 11-bit exponent
        let exponent = ((bits >> 52) & 0x7FF) as i32;
        // Bias is 1023 for f64
        let mut exponent_value = exponent - 1023;
        // 52-bit significand (add implicit 1 at the beginning)
        let significand_value: u64 = (bits & 0xFFFFFFFFFFFFF) | (1 << 52);

        let mut scaled = 0;
        let mut i = 1 << 52;
        let mut mask = 0xFFFFFFFFFFFFFu64;

        while exponent_value >= 0 {
            if (significand_value & i) != 0 {
                scaled += EXEMPTION_THRESHOLD_SCALE_FACTOR << exponent_value;
            }
            exponent_value -= 1;
            i >>= 1;
            mask >>= 1;
        }

        // reset the exponent value to be positive
        exponent_value = 1;
        // Move the mask back to the first bit after the decimal point.
        mask <<= 1;

        while significand_value & mask != 0 {
            if (significand_value & i) != 0 {
                scaled += EXEMPTION_THRESHOLD_SCALE_FACTOR / (1u64 << exponent_value);
            }
            exponent_value += 1;
            i >>= 1;
            mask >>= 1;
        }

        scaled
    }
}

impl Sysvar for Rent {
    impl_sysvar_get!(sol_get_rent_sysvar);
}

/// The return value of [`Rent::due`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RentDue {
    /// Used to indicate the account is rent exempt.
    Exempt,
    /// The account owes this much rent.
    Paying(u64),
}

impl RentDue {
    /// Return the lamports due for rent.
    pub fn lamports(&self) -> u64 {
        match self {
            RentDue::Exempt => 0,
            RentDue::Paying(x) => *x,
        }
    }

    /// Return 'true' if rent exempt.
    pub fn is_exempt(&self) -> bool {
        match self {
            RentDue::Exempt => true,
            RentDue::Paying(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sysvars::rent::{
        DEFAULT_BURN_PERCENT, DEFAULT_EXEMPTION_THRESHOLD, DEFAULT_LAMPORTS_PER_BYTE_YEAR,
    };

    #[test]
    pub fn test_minimum_balance() {
        let rent = super::Rent {
            lamports_per_byte_year: DEFAULT_LAMPORTS_PER_BYTE_YEAR,
            exemption_threshold: DEFAULT_EXEMPTION_THRESHOLD,
            burn_percent: DEFAULT_BURN_PERCENT,
        };

        assert_eq!(rent.minimum_balance(0), rent.minimum_balance_scaled(0));
        assert_eq!(rent.minimum_balance(100), rent.minimum_balance_scaled(100));
        assert_eq!(
            rent.minimum_balance(1000),
            rent.minimum_balance_scaled(1000)
        );
    }
}
