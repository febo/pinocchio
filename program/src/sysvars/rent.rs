//! This account contains the current cluster rent
//!
//! This is required for the rent sysvar implementation.

use crate::impl_sysvar_get;
use super::Sysvar;

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

impl Sysvar for Rent {
    impl_sysvar_get!(sol_get_rent_sysvar);
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
    pub fn due(&self, bytes: u64, years: f64) -> u64 {
        (self.lamports_per_byte_year * bytes as u64)
            .saturating_mul((years * 100.0) as u64)
            .saturating_div(100)
    }

    /// Calculates the minimum balance for rent exemption
    ///
    /// # Arguments
    ///
    /// * `bytes` - The number of bytes in the account
    ///
    /// # Returns
    ///
    /// The minimum balance in lamports for rent exemption
    pub fn minimum_balance(&self, bytes: u64) -> u64 {
        self.due(bytes, self.exemption_threshold)
    }

    /// Determines if an account can be considered rent exempt
    ///
    /// # Arguments
    ///
    /// * `lamports` - The balance of the account in lamports
    /// * `bytes` - The size of the account in bytes
    ///
    /// # Returns
    ///
    /// true if the account is rent exempt, false otherwise
    pub fn is_exempt(&self, lamports: u64, bytes: u64) -> bool {
        lamports >= self.minimum_balance(bytes)
    }
}
