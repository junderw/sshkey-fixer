//! SSH key fixer utility
//!
//! This utility processes SSH key files by:
//! 1. Reading an SSH private key file
//! 2. Decrypting if encrypted (prompting for password)
//! 3. Setting the UV REQUIRED flag (0x04) for SkEd25519 keys
//! 4. Writing the modified key back, re-encrypting if it was originally encrypted

use ssh_key::private::KeypairData;
use ssh_key::PrivateKey;
use std::fs;
use std::io::{self, Write};
use std::{env, process};

const UV_REQUIRED_FLAG: u8 = 0x04;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!(
            "Usage: {} <file_path>",
            args.first().unwrap_or(&"sshkey-fixer".to_string())
        );
        process::exit(1);
    }

    let file_path = &args[1];

    if let Err(e) = process_ssh_key(file_path) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    println!("SSH key successfully modified.");
}

fn process_ssh_key(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_contents = fs::read_to_string(file_path)?;
    let pk = PrivateKey::from_openssh(&file_contents)?;

    let password = if pk.is_encrypted() {
        println!("The private key is encrypted.");
        Some(prompt_password("Enter passphrase: ")?)
    } else {
        println!("The private key is not encrypted.");
        None
    };

    let decrypted_key = if let Some(pass) = &password {
        pk.decrypt(pass)?
    } else {
        pk
    };

    let modified_key = modify_key(decrypted_key)?;

    // Re-encrypt if the original was encrypted, then save
    let output = if let Some(pass) = &password {
        modified_key.encrypt(&mut rand::thread_rng(), pass)?
    } else {
        modified_key
    };

    // Preserve original line endings
    let line_ending = if file_contents.contains("\r\n") {
        ssh_key::LineEnding::CRLF
    } else if file_contents.contains("\r") {
        ssh_key::LineEnding::CR
    } else {
        ssh_key::LineEnding::LF
    };

    let openssh_output = output.to_openssh(line_ending)?;
    fs::write(file_path, openssh_output.as_bytes())?;

    Ok(())
}

fn prompt_password(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    rpassword::read_password()
}

fn modify_key(key: PrivateKey) -> Result<PrivateKey, Box<dyn std::error::Error>> {
    match key.key_data() {
        KeypairData::SkEd25519(sk_key) => {
            println!("Key type: SkEd25519");
            println!("Current flags: 0x{:02X}", sk_key.flags());

            if sk_key.flags() & UV_REQUIRED_FLAG != 0 {
                println!("UV_REQUIRED flag is already set. No changes needed.");
                return Ok(key);
            }

            // Clone and modify the key
            let mut modified_sk = sk_key.clone();
            modified_sk.set_flags(modified_sk.flags() | UV_REQUIRED_FLAG);
            // in hex
            println!("    New flags: 0x{:02X}", modified_sk.flags());

            // Rebuild the private key with modified data
            let new_key_data = KeypairData::SkEd25519(modified_sk);

            Ok(PrivateKey::new(new_key_data, key.comment().to_string())?)
        }
        KeypairData::SkEcdsaSha2NistP256(sk_key) => {
            println!("Key type: SkEcdsaSha2NistP256");
            println!("Current flags: 0x{:02X}", sk_key.flags());

            if sk_key.flags() & UV_REQUIRED_FLAG != 0 {
                println!("UV_REQUIRED flag is already set. No changes needed.");
                return Ok(key);
            }

            let mut modified_sk = sk_key.clone();
            modified_sk.set_flags(modified_sk.flags() | UV_REQUIRED_FLAG);
            println!("    New flags: 0x{:02X}", modified_sk.flags());

            let new_key_data = KeypairData::SkEcdsaSha2NistP256(modified_sk);

            Ok(PrivateKey::new(new_key_data, key.comment().to_string())?)
        }
        other => {
            Err(format!(
                "Unsupported key type: {}. This tool only supports SkEd25519 and SkEcdsaSha2NistP256 keys.",
                other.algorithm()?
            ).into())
        }
    }
}
