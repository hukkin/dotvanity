use clap;
use data_encoding::HEXLOWER;
use rand::rngs::OsRng;
use schnorrkel::Keypair;
use sp_core::crypto::AccountId32;
use sp_core::crypto::Ss58AddressFormat;
use sp_core::crypto::Ss58Codec;

fn is_valid_ss58_char(c: char) -> bool {
    let ss58_chars = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
        'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c',
        'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
        'w', 'x', 'y', 'z',
    ];
    ss58_chars.contains(&c)
}

struct Matcher {
    addr_type: u8,
    startswith: String,
    endswith: String,
}

impl Matcher {
    fn match_(&self, candidate: &String) -> bool {
        if !candidate.starts_with(&self.startswith) {
            return false;
        }
        if !candidate.ends_with(&self.endswith) {
            return false;
        }
        true
    }
    fn validate(&self) {
        if !self.startswith.chars().all(is_valid_ss58_char)
            || !self.endswith.chars().all(is_valid_ss58_char)
        {
            eprintln!("Error: A provided matcher contains SS58 incompatible characters");
            std::process::exit(1);
        }

        if self.addr_type == 0 && !self.startswith.starts_with("1") {
            eprintln!("Error: Polkadot mainnet address must start with \"1\". Adjust --startswith");
            std::process::exit(1);
        }
    }
}

fn generate_wallet(addr_format: u8) -> (Keypair, String) {
    let keypair: Keypair = Keypair::generate_with(OsRng);
    let account_id = AccountId32::from(keypair.public.to_bytes());
    let account_id_str =
        account_id.to_ss58check_with_version(Ss58AddressFormat::Custom(addr_format));
    (keypair, account_id_str)
}

fn main() {
    let matches = clap::App::new("dotvanity")
        .version("0.1.0")
        .author("Taneli Hukkinen <hukkinj1@users.noreply.github.com>")
        .about("Polkadot/Substrate vanity address generator")
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
        .get_matches();

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

    let matcher = Matcher {
        addr_type: addr_type,
        startswith: String::from(matches.value_of("startswith").unwrap()),
        endswith: String::from(matches.value_of("endswith").unwrap()),
    };
    matcher.validate();

    let mut wallet: (Keypair, String);
    loop {
        wallet = generate_wallet(addr_type);
        if matcher.match_(&wallet.1) {
            break;
        }
    }

    println!("Private key: {}", HEXLOWER.encode(&wallet.0.to_bytes()));
    println!(
        "Public key: {}",
        HEXLOWER.encode(&wallet.0.public.to_bytes())
    );
    println!("Address: {}", wallet.1);
}
