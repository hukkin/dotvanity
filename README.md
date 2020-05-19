[![Crate](https://img.shields.io/crates/v/dotvanity.svg)](https://crates.io/crates/dotvanity)
[![Build Status](https://travis-ci.com/hukkinj1/dotvanity.svg?branch=master)](https://travis-ci.com/hukkinj1/dotvanity)
# dotvanity

<!--- Don't edit the version line below manually. Let bump2version do it for you. -->
> Version 0.1.2

> CLI tool for generating [Substrate](https://substrate.dev/) (or [Polkadot](https://polkadot.network/)) vanity addresses


## Features
* Generate SS58 encoded vanity addresses
* Support address types 0 to 127 (includes Polkadot, Kusama, generic Substrate etc.). Defaults to Polkadot mainnet (address type 0).
* Use all CPU cores
* Specify a substring that the addresses must
    * start with
    * end with
* Output a corresponding BIP39 mnemonic phrase along with the address

## Installing
```bash
cargo install dotvanity
```

## Usage examples
Find an address that starts with "11" (e.g. 11Tvp5FaD2Vf69BS5tgGJio8KBPd6PUSvrn9nyDTCLWnQWw)
```bash
dotvanity --startswith 11
```

Find an address that ends with "zz" (e.g. 1X9fUsYxfJ3qJvGu9wdZNhaKP37Y9Vg1YgsMKgkrDox9Pzz)
```bash
dotvanity --endswith zz
```

Use 5 CPU threads. The default is 1.
```bash
dotvanity --cpus 5
```

Alter the address type. Create a Kusama address (type 2) instead of Polkadot.
```bash
dotvanity --type 2
```

Generate 5 addresses (the default is 1)
```bash
dotvanity -n 5
```

Combine flags introduced above
```bash
dotvanity --startswith 11 --endswith QQ --cpus 3
```
