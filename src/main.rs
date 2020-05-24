use data_encoding::HEXLOWER;
use sp_core::crypto::AccountId32;
use sp_core::crypto::Ss58AddressFormat;
use sp_core::crypto::Ss58Codec;
use sp_core::Pair;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

fn is_valid_ss58_char(c: char) -> bool {
    let ss58_chars = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
        'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c',
        'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
        'w', 'x', 'y', 'z',
    ];
    ss58_chars.contains(&c)
}

fn count_digits(string: &str) -> usize {
    string.chars().filter(|c| c.is_ascii_digit()).count()
}

fn count_letters(string: &str) -> usize {
    string.chars().filter(|c| c.is_ascii_alphabetic()).count()
}

#[derive(Clone)]
struct Matcher {
    addr_type: u8,
    startswith: String,
    endswith: String,
    contains: String,
    digits: Option<usize>,
    letters: Option<usize>,
}

impl Matcher {
    fn match_(&self, candidate: &str) -> bool {
        if !candidate.contains(&self.contains) {
            return false;
        }
        if !candidate.starts_with(&self.startswith) {
            return false;
        }
        if !candidate.ends_with(&self.endswith) {
            return false;
        }
        if let Some(digits) = self.digits {
            if count_digits(candidate) < digits {
                return false;
            }
        }
        if let Some(letters) = self.letters {
            if count_letters(candidate) < letters {
                return false;
            }
        }
        true
    }

    /// Validates the current configuration
    fn validate(&self) -> Result<(), &str> {
        if !self.startswith.chars().all(is_valid_ss58_char)
            || !self.endswith.chars().all(is_valid_ss58_char)
            || !self.contains.chars().all(is_valid_ss58_char)
        {
            return Err("Error: A provided matcher contains SS58 incompatible characters");
        }

        // Validate first char of --startswith string for some known cases
        if !self.startswith.is_empty() {
            let first_char = self.startswith.chars().next().unwrap();
            if self.addr_type == 0 && first_char != '1' {
                return Err(
                    "Error: Polkadot mainnet address must start with '1'. Adjust --startswith",
                );
            }
            let kusama_addr_first_chars = ['C', 'D', 'F', 'G', 'H', 'J'];
            if self.addr_type == 2 && !kusama_addr_first_chars.contains(&first_char) {
                return Err("Error: Kusama address must start with one of ['C', 'D', 'F', 'G', 'H', 'J']. Adjust --startswith");
            }
            if self.addr_type == 42 && first_char != '5' {
                return Err(
                    "Error: Generic Substrate address must start with '5'. Adjust --startswith",
                );
            }
        }
        Ok(())
    }
}

struct Wallet {
    mnemonic_phrase: String,
    private_key: [u8; 32],
    public_key: [u8; 32],
    address: String,
}

impl Wallet {
    fn new(addr_format: u8, with_phrase: bool) -> Wallet {
        if with_phrase {
            return Wallet::with_phrase(addr_format);
        }
        Wallet::without_phrase(addr_format)
    }

    fn with_phrase(addr_format: u8) -> Wallet {
        let (pair, phrase, secret) = sp_core::sr25519::Pair::generate_with_phrase(None);
        let address = AccountId32::from(pair.public())
            .to_ss58check_with_version(Ss58AddressFormat::Custom(addr_format));
        Wallet {
            mnemonic_phrase: phrase,
            private_key: secret,
            public_key: <[u8; 32]>::from(pair.public()),
            address,
        }
    }

    fn without_phrase(addr_format: u8) -> Wallet {
        let phrase = String::new();
        let (pair, secret) = sp_core::sr25519::Pair::generate();
        let address = AccountId32::from(pair.public())
            .to_ss58check_with_version(Ss58AddressFormat::Custom(addr_format));
        Wallet {
            mnemonic_phrase: phrase,
            private_key: secret,
            public_key: <[u8; 32]>::from(pair.public()),
            address,
        }
    }

    fn pretty_print(&self) {
        if !self.mnemonic_phrase.is_empty() {
            println!("Mnemonic phrase: {}", self.mnemonic_phrase);
        }
        println!("Private key:     {}", HEXLOWER.encode(&self.private_key));
        println!("Public key:      {}", HEXLOWER.encode(&self.public_key));
        println!("Address:         {}", self.address);
    }
}

// Generate wallets and send matching wallets to `tx` until `kill_pill`
// is received.
fn generate_matching_wallet(
    tx: Sender<Wallet>,
    attempt_count_tx: Sender<u64>,
    kill_pill: Receiver<()>,
    matcher: Matcher,
    addr_type: u8,
    with_phrase: bool,
) {
    let mut unreported_attempts: u64 = 0;
    let mut wallet: Wallet;
    loop {
        wallet = Wallet::new(addr_type, with_phrase);
        if matcher.match_(&wallet.address) {
            tx.send(wallet).unwrap();
        }

        let report_threshold = 1000; // Report attempt count to main thread after this many attempts
        unreported_attempts += 1;
        if unreported_attempts >= report_threshold {
            attempt_count_tx.send(unreported_attempts).unwrap();
            unreported_attempts = 0;
        }

        match kill_pill.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
    }
}

fn main() {
    let matches = clap::App::new("dotvanity")
        .version("0.2.7")  // DO NOT EDIT THIS LINE MANUALLY
        .author("Taneli Hukkinen <hukkinj1@users.noreply.github.com>")
        .about("Polkadot/Substrate vanity address generator")
        .arg(
            clap::Arg::with_name("digits")
                .short("d")
                .long("digits")
                .value_name("INT")
                .help("Amount of digits (0-9) that the address must contain")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("letters")
                .short("l")
                .long("letters")
                .value_name("INT")
                .help("Amount of letters (a-z) that the address must contain")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("contains")
                .short("c")
                .long("contains")
                .value_name("SUBSTRING")
                .help("A string that the address must contain")
                .default_value(""),
        )
        .arg(
            clap::Arg::with_name("startswith")
                .short("s")
                .long("startswith")
                .value_name("SUBSTRING")
                .help("A string that the address must start with")
                .default_value(""),
        )
        .arg(
            clap::Arg::with_name("endswith")
                .short("e")
                .long("endswith")
                .value_name("SUBSTRING")
                .help("A string that the address must end with")
                .default_value(""),
        )
        .arg(
            clap::Arg::with_name("address type")
                .short("t")
                .long("type")
                .value_name("INT")
                .help("Address type. Should be an integer value in range 0 to 127.\n\
                          Notable types:\n\
                          \t0 - Polkadot mainnet\n\
                          \t2 - Kusama network\n\
                          \t42 - Generic Substrate\n\
                          Defaults to Polkadot mainnet. For more types, see \
                          https://github.com/paritytech/substrate/wiki/External-Address-Format-(SS58)#address-type")
                .default_value("0"),  // Polkadot mainnet type
        )
        .arg(
            clap::Arg::with_name("cpus")
                .long("cpus")
                .value_name("INT")
                .help("Amount of CPU cores to use")
                .default_value("1"),
        )
        .arg(
            clap::Arg::with_name("wallet count")
                .short("n")
                .long("count")
                .value_name("INT")
                .help("Amount of matching wallets to find")
                .default_value("1"),
        )
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Verbose output")
        )
        .arg(
            clap::Arg::with_name("mnemonic")
                .short("m")
                .long("mnemonic")
                .help("Generate a mnemonic phrase for wallets")
        )
        .get_matches();

    let mnemonic = match matches.occurrences_of("mnemonic") {
        0 => false,
        1 => true,
        _ => panic!("got more than one mnemonic flag"),
    };

    let verbose = match matches.occurrences_of("verbose") {
        0 => false,
        1 => true,
        _ => panic!("got more than one verbose"),
    };

    let wallet_count_str = matches.value_of("wallet count").unwrap();
    let wallet_count: u32 = match wallet_count_str.parse() {
        Ok(wallet_count) => wallet_count,
        Err(_error) => {
            eprintln!("Error: Wallet count is not a 32-bit unsigned integer");
            std::process::exit(1);
        }
    };

    let cpu_count_str = matches.value_of("cpus").unwrap();
    let cpu_count: u8 = match cpu_count_str.parse() {
        Ok(cpu_count) => cpu_count,
        Err(_error) => {
            eprintln!("Error: CPU count is not an 8-bit unsigned integer");
            std::process::exit(1);
        }
    };

    let addr_type_str = matches.value_of("address type").unwrap();
    let addr_type: u8 = match addr_type_str.parse() {
        Ok(addr_type) => addr_type,
        Err(_error) => {
            eprintln!("Error: Address type is not an 8-bit unsigned integer");
            std::process::exit(1);
        }
    };
    if addr_type > 127 {
        eprintln!("Error: Address type must be in range [0, 127]");
        std::process::exit(1);
    }

    let digit_count = match matches.value_of("digits") {
        None => None,
        Some(count_str) => match count_str.parse() {
            Ok(c) => Some(c),
            Err(_) => {
                eprintln!("Error: Digit count is not a valid integer");
                std::process::exit(1);
            }
        },
    };
    if let Some(count) = digit_count {
        if count > 48 {
            eprintln!("Error: Digit count must be in range [0, 48]");
            std::process::exit(1);
        }
    }

    let letter_count = match matches.value_of("letters") {
        None => None,
        Some(count_str) => match count_str.parse() {
            Ok(c) => Some(c),
            Err(_) => {
                eprintln!("Error: Letter count is not a valid integer");
                std::process::exit(1);
            }
        },
    };
    if let Some(count) = letter_count {
        if count > 48 {
            eprintln!("Error: Letter count must be in range [0, 48]");
            std::process::exit(1);
        }
    }

    let matcher = Matcher {
        addr_type,
        startswith: String::from(matches.value_of("startswith").unwrap()),
        endswith: String::from(matches.value_of("endswith").unwrap()),
        contains: String::from(matches.value_of("contains").unwrap()),
        digits: digit_count,
        letters: letter_count,
    };

    if let Err(error) = matcher.validate() {
        eprintln!("{}", error);
        std::process::exit(1);
    }

    let (tx, rx) = mpsc::channel();
    let (attempt_count_tx, attempt_count_rx) = mpsc::channel();
    let mut children = Vec::new();
    let mut kill_pills = Vec::new();
    for _child_index in 0..cpu_count {
        let thread_tx = tx.clone();
        let thread_attempt_count_tx = attempt_count_tx.clone();
        let thread_matcher = matcher.clone();
        let (kill_pill_tx, kill_pill_rx) = mpsc::channel();
        let child = thread::spawn(move || {
            generate_matching_wallet(
                thread_tx,
                thread_attempt_count_tx,
                kill_pill_rx,
                thread_matcher,
                addr_type,
                mnemonic,
            )
        });
        kill_pills.push(kill_pill_tx);
        children.push(child);
    }

    let start_time = SystemTime::now();
    let mut matches_found = 0;
    let mut total_attempts: u64 = 0;
    while matches_found < wallet_count {
        match rx.recv_timeout(Duration::from_secs(3)) {
            Ok(matching_wallet) => {
                matches_found += 1;
                println!(":::: Matching wallet found ::::");
                matching_wallet.pretty_print();
            }
            Err(RecvTimeoutError::Disconnected) => panic!("wallet tx disconnected"),
            Err(RecvTimeoutError::Timeout) => {}
        }

        // Read the attempt_count channel until it's empty
        total_attempts += attempt_count_rx.try_iter().sum::<u64>();

        if verbose {
            if let Ok(elapsed) = start_time.elapsed() {
                let elapsed_secs = elapsed.as_secs();
                if elapsed_secs != 0 {
                    println!(
                        "Pace: {} attempts per second",
                        total_attempts / elapsed.as_secs()
                    )
                }
            }
        }
    }

    // Tear down all child threads
    for pill in kill_pills {
        pill.send(()).unwrap();
    }
    for child in children {
        child.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_startswith_first_char() {
        let m = Matcher {
            addr_type: 0,
            startswith: String::from("1"),
            endswith: String::new(),
            contains: String::new(),
            letters: None,
            digits: None,
        };
        assert!(m.validate().is_ok());
    }

    #[test]
    fn test_incorrect_startswith_first_char() {
        let m = Matcher {
            addr_type: 0,
            startswith: String::from("2"),
            endswith: String::new(),
            contains: String::new(),
            letters: None,
            digits: None,
        };
        assert_eq!(
            m.validate(),
            Err("Error: Polkadot mainnet address must start with '1'. Adjust --startswith")
        );
    }
}
