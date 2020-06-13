[![Crate](https://img.shields.io/crates/v/dotvanity.svg)](https://crates.io/crates/dotvanity)
[![Build Status - Travis](https://travis-ci.com/hukkinj1/dotvanity.svg?branch=master)](https://travis-ci.com/hukkinj1/dotvanity)
[![Build status- AppVeyor](https://ci.appveyor.com/api/projects/status/xgyd0s7vo2va9dh7/branch/master?svg=true)](https://ci.appveyor.com/project/hukkinj1/dotvanity/branch/master)
# dotvanity

<!--- Don't edit the version line below manually. Let bump2version do it for you. -->
> Version 1.0.0

> CLI tool for generating [Substrate](https://substrate.dev/) (or [Polkadot](https://polkadot.network/)) vanity addresses


## Features
* Generate SS58 encoded vanity addresses using sr25519 keypairs
* Support address types 0 to 127 (includes Polkadot, Kusama, generic Substrate etc.). Defaults to Polkadot mainnet (address type 0).
* Specify number of CPU cores used
* Specify a substring that the addresses must
    * start with
    * end with
    * contain
* Set a required minimum amount of letters (a-z or A-Z) or digits (0-9) that the address must contain
* Output a corresponding BIP39 mnemonic phrase along with the address
* Binaries built for Linux, macOS and Windows

## Installing
Download the latest binary release from the [_Releases_](https://github.com/hukkinj1/dotvanity/releases) page.

Alternatively, if you have `cargo` installed, build and install by running
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

Find an address containing the substring "XXXXX" (e.g. 1R6DVtPBh5ZfNHPFoHT4GVUuLwzcbZaVvD4EFXXXXXZMBc3)
```bash
dotvanity --contains XXXXX
```

Find an address with at least 46 letters (e.g. 14KhqiucsPQJYfBQnYYUMTKSNUjwFdFzFGEMyjEUedCpJSFa)
```bash
dotvanity --letters 46
```

Find an address with at least 25 digits (e.g. 148GwY3868mW4vGvrQtq4266CK3165835N593ngW9B57HDBg)
```bash
dotvanity --digits 25
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

Output a BIP39 mnemonic phrase for found addresses. **NOTE: This is resource intensive and makes finding an address a LOT slower.**
```bash
dotvanity --mnemonic
```

Combine flags introduced above
```bash
dotvanity --startswith 11 --endswith QQ --cpus 3
```
