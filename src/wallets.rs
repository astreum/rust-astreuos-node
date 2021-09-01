
extern crate rand;

use rand::RngCore;
use rand::rngs::OsRng;

use sha3::{Digest, Sha3_256};

use ring::{digest, pbkdf2};
use std::num::NonZeroU32;

use std::error::Error;

use stellar_notation::{
    StellarObject, StellarValue
};

use crate::library::wordlist;

pub fn create() -> Result<(), Box<dyn Error>> {

    let mut store = neutrondb::store("app")?;

    let master_key_query = store.get("master_key")?;

    match master_key_query {
        
        Some(_) => {

            print!(r###"

    Wallet already created!
            "###);
            
            Ok(())
        },

        None => {

            print!(r###"

    Creating Wallet ...
            "###);

            print!(r###"

    Generating Entropy ...
            "###);

            let mut entropy = vec![0u8; 32];

            OsRng.fill_bytes(&mut entropy);

            let mut hasher = Sha3_256::new();

            hasher.update(&entropy);

            let hash = hasher.finalize();

            let entropy_checksum = [entropy, vec![hash[0]]].concat();

            let mut bits = "".to_string();

            for byte in entropy_checksum {

                let b_rep = &format!("00000000{:b}", byte);
                let len = b_rep.len();
                bits += &b_rep[len-8..len];

            }

            print!(r###"

    Generating Mnemonic ...
            "###);

            let mnemonic_length: Vec<usize> = (0..24).collect();

            let phrase: Vec<String> = mnemonic_length.iter()
                .map(|x| {

                    let first_bit = x * 11;
                    let slice = &bits[first_bit..first_bit + 11];
                    let int = usize::from_str_radix(slice, 2).unwrap();
                    wordlist::get(int)

                })
                .collect();

            let mnemonic_art = format!(
                r###"

        {}  {}  {}  {}  {}  {}

        {}  {}  {}  {}  {}  {}

        {}  {}  {}  {}  {}  {}

        {}  {}  {}  {}  {}  {}

                "###,
                phrase[0], phrase[1], phrase[2], phrase[3], phrase[4], phrase[5],
                phrase[6], phrase[7], phrase[8], phrase[9], phrase[10], phrase[11],
                phrase[12], phrase[13], phrase[14], phrase[15], phrase[16], phrase[17],
                phrase[18], phrase[19], phrase[20], phrase[21], phrase[22], phrase[23]
            );

            print!("{}", mnemonic_art);

            let mnemonic_string: String = phrase.concat();

            let master_key = seed_to_master(mnemonic_string)?;

            let master_key_object: StellarObject = StellarObject(
                "master_key".to_string(),
                StellarValue::Bytes(master_key)
            );
        
            store.put(master_key_object)?;

            Ok(())

        }
    
    }

}

// pub fn recover() {}

pub fn remove() -> Result<(), Box<dyn Error>> {

    print!(r###"

    Removing Wallet ...
    "###);

    let mut store = neutrondb::store("app")?;

    store.delete("master_key")?;

    print!(r###"

    Removed!
    "###);

    Ok(())
}

// pub fn show() {}

pub fn seed_to_master(seed: String) -> Result<Vec<u8>, Box<dyn Error>> {

    let salt = "mnemonic";

    let mut pbkdf2_hash: [u8; digest::SHA512_OUTPUT_LEN] = [0u8; digest::SHA512_OUTPUT_LEN];

    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        NonZeroU32::new(2_048).unwrap(),
        salt.as_bytes(),
        seed.as_bytes(),
        &mut pbkdf2_hash
    );

    Ok(pbkdf2_hash.to_vec())

}