//! # Vehicle Identification Number parser
//!
//! Parser and checksum verifier for VIN.
//!
//! Provides information about region, manufacturer, country of origin, possible years of assembling
//! and checksum validation of given Vehicle Identification Number.
//!
//! # Examples
//! ```
//! // Check whether VIN is ok (without checksum validation)
//! let vin_number = "WP0ZZZ99ZTS392124";
//! assert!(vin::check_validity(vin_number).is_ok());
//! ```
//!
//! ```
//! // Check VIN with checksum validation
//! let vin_number = "1M8GDM9AXKP042788";
//! assert!(vin::verify_checksum(vin_number).is_ok());
//! ```
//!
//! ```
//! // Get VIN information
//! let vin_number = "wp0zzz998ts392124";
//! let result = vin::get_info(vin_number).unwrap();
//! assert_eq!(result.vin, vin_number.to_uppercase());
//! assert_eq!(result.country, "Germany/West Germany");
//! assert_eq!(result.manufacturer, "Porsche car");
//! assert_eq!(result.region, "Europe");
//! assert!(result.valid_checksum.is_ok());
//! ```
#[macro_use]
extern crate lazy_static;

use std::collections::HashSet;
use std::fmt;
use std::time::SystemTime;

use crate::VINError::{ChecksumError, IncorrectLength, InvalidCharacters};
use crate::dicts::{get_region, get_country, get_manufacturer};

mod dicts;


/// Provides information about invalid checksum calculation from the VIN
#[derive(Debug, Copy, Clone)]
pub struct ChecksumErrorInfo {
    /// Expected symbol at the 9-nth place
    pub expected: char,

    /// Received symbol at the 9-th place
    pub received: char,
}

/// Provides possible errors during VIN parsing
#[derive(Debug)]
pub enum VINError {
    /// Provided number length != 17
    IncorrectLength,

    /// Provided number contains invalid characters
    InvalidCharacters(HashSet<char>),

    /// Provided number did not pass checksum validation (notice, that only North American VINs
    /// must pass this validation, for others it is not obligatory)
    ChecksumError(ChecksumErrorInfo),
}

impl fmt::Display for VINError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VINError::IncorrectLength =>
                write!(f, "Incorrect length of given string, 17 chars expected."),
            VINError::InvalidCharacters(chars) =>
                write!(f, "Invalid characters received in given string: {:?}.", chars),
            VINError::ChecksumError(err) =>
                write!(f, "Invalid checksum symbol on 9th place, {} expected, {} received.", err.expected, err.received),
        }
    }
}

/// Holds parsed information about the vehicle
#[derive(Debug, Clone)]
pub struct VIN {
    /// Copy of provided VIN number
    pub vin: String,

    /// Country of the manufacturer
    pub country: String,

    /// Name of the manufacturer
    pub manufacturer: String,

    /// Region of the manufacturer
    pub region: String,

    /// Whether checksum of the VIN is valid
    pub valid_checksum: Result<(), ChecksumErrorInfo>,
}


impl VIN {
    /// Returns WMI part of VIN
    pub fn wmi(&self) -> &str { &self.vin[..3] }

    /// Returns VDS part of VIN
    pub fn vds(&self) -> &str { &self.vin[3..9] }

    /// Returns VIS part of VIN
    pub fn vis(&self) -> &str { &self.vin[9..] }

    /// Returns whether manufacturer is small and does not have its own ID in VIN
    pub fn small_manufacturer(&self) -> bool { &self.wmi()[2..] == "9" }

    /// Returns region VIN code
    pub fn region_code(&self) -> &str { &self.wmi()[..1] }

    /// Returns country VIN code
    pub fn country_code(&self) -> &str { &self.wmi()[1..] }

    /// Returns possible years of assembling
    pub fn years(&self) -> Vec<u32> {
        let letters = "ABCDEFGHJKLMNPRSTVWXY123456789";
        let year_letter = &self.vis().chars().nth(0).unwrap();

        let mut year: u32 = 1979;
        let cur_year = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        let cur_year = cur_year / 3600.0 / 24.0 / 365.25 + 1970.0;  // get year
        let cur_year = (cur_year.round() + 2.0) as u32;             // add 2 years in advance

        let mut result = vec![];
        for letter in letters.chars().cycle() {
            year += 1;

            if letter == *year_letter {
                result.push(year);
            }

            if year == cur_year { break; }
        }

        result
    }
}


/// Validates Vehicle Identification Number without computing the checksum
/// (check used symbols and length of the number)
///
/// # Examples
/// ```
/// let vin_number = "WP0ZZZ99ZTS392124";
/// assert!(vin::check_validity(vin_number).is_ok());
///
/// let vin_number = "W$0ZZZ99ZTS392124";  // notice $ sign on 2-nd place
/// assert!(vin::check_validity(vin_number).is_err())
/// ```
pub fn check_validity(vin: &str) -> Result<(), VINError> {
    let vin = vin.to_uppercase();

    // check length
    if vin.chars().count() != 17 {
        return Err(IncorrectLength);
    }

    // check alphabet
    let used_chars: HashSet<char> = vin.chars().collect();
    let odd_chars: HashSet<char> = used_chars.difference(&dicts::ALLOWED_CHARS).cloned().collect();
    if odd_chars.len() > 0 {
        return Err(InvalidCharacters(odd_chars));
    }

    Ok(())
}


/// Validates Vehicle Identification Number AND validates the checksum
///
/// # Examples
/// ```
/// let vin_number = "1M8GDM9AXKP042788";
/// assert!(vin::verify_checksum(vin_number).is_ok());
///
/// let vin_number = "WP0ZZZ99ZTS392124";
/// assert!(match vin::verify_checksum(vin_number) {
///     Err(vin::VINError::ChecksumError(vin::ChecksumErrorInfo {
///         expected: '8',
///         received: 'Z',
///     })) => true,
///     _ => false,
/// })
/// ```
pub fn verify_checksum(vin: &str) -> Result<(), VINError> {
    let vin = vin.to_uppercase();
    check_validity(&vin)?;

    // verify checksum
    let checksum: u32 = vin
        .chars()
        .map(|x| dicts::VALUE_MAP.get(&x).unwrap())
        .zip(dicts::WEIGHTS.iter())
        .map(|(l, r)| l * r)
        .sum();


    let checknumber = match checksum % 11 {
        10 => 'X',
        i => std::char::from_digit(i, 10).unwrap()
    };

    let pr_number = vin.chars().nth(8).unwrap();
    if pr_number == checknumber {
        Ok(())
    } else {
        Err(ChecksumError(ChecksumErrorInfo {
            expected: checknumber,
            received: pr_number,
        }))
    }
}


/// Return basic information about manufacturer of the vehicle
///
/// # Examples
/// ```
/// let vin_number = "wp0zzz998ts392124";
/// let result = vin::get_info(vin_number).unwrap();
/// assert_eq!(result.vin, vin_number.to_uppercase());
/// assert_eq!(result.country, "Germany/West Germany");
/// assert_eq!(result.manufacturer, "Porsche car");
/// assert_eq!(result.region, "Europe");
/// assert!(result.valid_checksum.is_ok())
/// ```
pub fn get_info(vin: &str) -> Result<VIN, VINError> {
    let vin = vin.to_uppercase();
    check_validity(&vin)?;

    return Ok(VIN {
        vin: vin.clone(),
        country: get_country(&vin[..2]),
        manufacturer: get_manufacturer(&vin[..3]),
        region: get_region(&vin[..1]),
        valid_checksum: match verify_checksum(&vin) {
            Ok(()) => Ok(()),
            Err(VINError::ChecksumError(x)) => Err(x),
            _ => Ok(())     // unreachable
        }
    })
}


