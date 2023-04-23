# VIN
[![Build Status](https://travis-ci.org/maybe-hello-world/vin.svg?branch=master)](https://travis-ci.org/maybe-hello-world/vin)
[![codecov](https://codecov.io/gh/maybe-hello-world/vin/branch/master/graph/badge.svg)](https://codecov.io/gh/maybe-hello-world/vin)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![docs: latest](https://docs.rs/vin/badge.svg)](https://docs.rs/vin)



Vehicle Identification Number (VIN) parser and validator for Rust.

Provides information about region, manufacturer, country of origin, possible years of assembling
and checksum validation of given Vehicle Identification Number.
 
## Examples
Add dependency to your `Cargo.toml`
```
[dependencies]
vin_parser = "1.0.0"
```
Then, in your crate:
```rust
extern crate vin;

let vin_number = "WP0ZZZ99ZTS392124";
assert!(vin::check_validity(vin_number).is_ok());
```

```rust
extern crate vin;

// Check VIN with checksum validation
let vin_number = "1M8GDM9AXKP042788";
assert!(vin::verify_checksum(vin).is_ok());
```

```rust
extern crate vin;

// Get VIN information
let vin_number = "wp0zzz998ts392124";
let result = vin::get_info(vin_number).unwrap();
assert_eq!(result.vin, vin_number.to_uppercase());
assert_eq!(result.country, "Germany/West Germany");
assert_eq!(result.manufacturer, "Porsche car");
assert_eq!(result.region, "Europe");
assert!(result.valid_checksum.is_ok());
```

## Thanks
Inspired by this repository: https://github.com/idlesign/vininfo.
