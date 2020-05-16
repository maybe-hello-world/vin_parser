extern crate vin;

use vin::{check_validity, get_info, verify_checksum, VINError};

#[test]
fn check_length() {
    let erroneous = check_validity("");
    assert!(erroneous.is_err() && match erroneous.unwrap_err() {
        VINError::IncorrectLength => true,
        _ => false
    });

    let valid = check_validity("00000000000000000");
    assert!(valid.is_ok())
}

#[test]
fn check_alphabet() {
    let erroneous = check_validity("abcdefghioq_958.!");
    assert!(erroneous.is_err() && match erroneous.unwrap_err() {
        VINError::InvalidCharacters(_) => true,
        _ => false
    });

    let valid = check_validity("0123456789abcdefg");
    assert!(valid.is_ok())
}

#[test]
fn checksum() {
    let erroneous = verify_checksum("WP0ZZZ99ZTS392124");
    assert!(match erroneous.unwrap_err() {
        vin::VINError::ChecksumError(vin::ChecksumErrorInfo {
                                         expected: '8',
                                         received: 'Z',
                                     }) => true,
        _ => false,
    });

    let valid = verify_checksum("1M8GDM9AXKP042788");
    assert!(valid.is_ok())
}

#[test]
fn test_info() {
    let vin = "WP0ZZZ99ZTS392124";
    let result = get_info(vin);

    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.vin, vin);
    assert_eq!(result.country, "Germany/West Germany");
    assert_eq!(result.manufacturer, "Porsche car");
    assert_eq!(result.region, "Europe");
    assert!(match result.valid_checksum {
        Err(info) => (info.expected == '8' && info.received == 'Z'),
        Ok(_) => false
    });
}
