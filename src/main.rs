use ssh_key::private::KeypairData;
use ssh_key::PrivateKey;
use std::fs;
use std::io::{self, Write};
use std::{env, process};

// Require user presence
const UP_REQUIRED_FLAG: u8 = 0x01;
// Require user verification
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

    if let Err(e) = run_interactive(file_path) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_interactive(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_contents = fs::read_to_string(file_path)?;
    let pk = PrivateKey::from_openssh(&file_contents)?;

    let original_password = if pk.is_encrypted() {
        println!("The private key is encrypted.");
        Some(prompt_password("Enter passphrase: ")?)
    } else {
        println!("The private key is not encrypted.");
        None
    };

    let decrypted_key = if let Some(pass) = &original_password {
        pk.decrypt(pass)?
    } else {
        pk
    };

    // Get current flags
    let (key_type, current_flags) = get_key_info(&decrypted_key)?;

    let mut flags = current_flags;
    let mut message: Option<&str> = Some("Use the menu to toggle flags and save the key.");

    loop {
        clear_screen();
        println!("SSH Key Flag Editor");
        println!("===================");
        println!();
        println!("File: {}", file_path);
        println!("Key type: {}", key_type);
        println!();
        display_flags(flags);
        println!();
        println!("1. Toggle User Presence (UP)");
        println!("2. Toggle User Verification (UV)");
        println!("3. Save without password");
        if original_password.is_some() {
            println!("4. Save with same password");
        }
        println!("5. Save with new password");
        println!("6. Quit without saving");
        println!();

        if let Some(msg) = message {
            println!("{}", msg);
            println!();
        } else {
            println!();
            println!();
        }

        let choice = prompt_input("Enter choice: ")?;

        match choice.trim() {
            "1" => {
                flags ^= UP_REQUIRED_FLAG;
                message = Some("Toggled User Presence flag.");
            }
            "2" => {
                flags ^= UV_REQUIRED_FLAG;
                message = Some("Toggled User Verification flag.");
            }
            "3" => {
                save_key(file_path, &file_contents, &decrypted_key, flags, None)?;
                clear_screen();
                println!("SSH key saved without password.");
                break;
            }
            "4" if original_password.is_some() => {
                save_key(
                    file_path,
                    &file_contents,
                    &decrypted_key,
                    flags,
                    original_password.as_deref(),
                )?;
                clear_screen();
                println!("SSH key saved with same password.");
                break;
            }
            "5" => {
                let new_pass = prompt_password("Enter new passphrase: ")?;
                let confirm_pass = prompt_password("Confirm new passphrase: ")?;
                if new_pass != confirm_pass {
                    message = Some("Passphrases do not match. Try again.");
                    continue;
                }
                save_key(
                    file_path,
                    &file_contents,
                    &decrypted_key,
                    flags,
                    Some(&new_pass),
                )?;
                clear_screen();
                println!("SSH key saved with new password.");
                break;
            }
            "6" => {
                clear_screen();
                println!("Exiting without saving.");
                break;
            }
            _ => {
                message = Some("Invalid choice. Please try again.");
            }
        }
    }

    Ok(())
}

fn clear_screen() {
    // ANSI escape codes: clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().unwrap();
}

fn display_flags(flags: u8) {
    let up_checked = if flags & UP_REQUIRED_FLAG != 0 {
        "*"
    } else {
        " "
    };
    let uv_checked = if flags & UV_REQUIRED_FLAG != 0 {
        "*"
    } else {
        " "
    };

    println!("Current flags:");
    println!("  [{}] User Presence Required (UP)", up_checked);
    println!("  [{}] User Verification Required (UV)", uv_checked);
}

fn get_key_info(key: &PrivateKey) -> Result<(&'static str, u8), Box<dyn std::error::Error>> {
    match key.key_data() {
        KeypairData::SkEd25519(sk_key) => Ok(("SkEd25519", sk_key.flags())),
        KeypairData::SkEcdsaSha2NistP256(sk_key) => Ok(("SkEcdsaSha2NistP256", sk_key.flags())),
        other => Err(format!(
            "Unsupported key type: {}. This tool only supports SkEd25519 and SkEcdsaSha2NistP256 keys.",
            other.algorithm()?
        )
        .into()),
    }
}

fn save_key(
    file_path: &str,
    original_contents: &str,
    key: &PrivateKey,
    new_flags: u8,
    password: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let modified_key = set_key_flags(key, new_flags)?;

    let output = if let Some(pass) = password {
        modified_key.encrypt(&mut rand::thread_rng(), pass)?
    } else {
        modified_key
    };

    // Preserve original line endings
    let line_ending = if original_contents.contains("\r\n") {
        ssh_key::LineEnding::CRLF
    } else if original_contents.contains("\r") {
        ssh_key::LineEnding::CR
    } else {
        ssh_key::LineEnding::LF
    };

    let openssh_output = output.to_openssh(line_ending)?;
    fs::write(file_path, openssh_output.as_bytes())?;

    Ok(())
}

fn set_key_flags(
    key: &PrivateKey,
    new_flags: u8,
) -> Result<PrivateKey, Box<dyn std::error::Error>> {
    match key.key_data() {
        KeypairData::SkEd25519(sk_key) => {
            let mut modified_sk = sk_key.clone();
            modified_sk.set_flags(new_flags);
            let new_key_data = KeypairData::SkEd25519(modified_sk);
            Ok(PrivateKey::new(new_key_data, key.comment().to_string())?)
        }
        KeypairData::SkEcdsaSha2NistP256(sk_key) => {
            let mut modified_sk = sk_key.clone();
            modified_sk.set_flags(new_flags);
            let new_key_data = KeypairData::SkEcdsaSha2NistP256(modified_sk);
            Ok(PrivateKey::new(new_key_data, key.comment().to_string())?)
        }
        other => Err(format!("Unsupported key type: {}.", other.algorithm()?).into()),
    }
}

fn prompt_password(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    rpassword::read_password()
}

fn prompt_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}
