# SSH Key Fixer

A command-line interactive utility written in Rust that allows you to view and toggle the User Presence (UP) and User Verification (UV) flags for OpenSSH FIDO2/U2F keys, then save the key with or without a password.

## Overview

There was an issue in openssh when using `ssh-keygen -K` to output the ssh key files for security keys. It turns out that UV REQ flag was not being properly output into the key file when the security key has internal user verification (like Yubikey BIO).

https://github.com/Yubico/libfido2/discussions/926#discussioncomment-15523431


This discussion explains the issue. This tool lets you interactively view and set the UP and UV flags for an affected key file, then save it in-place with or without a password.

**Supported key types:**
- `SkEd25519` (FIDO2 keys using Ed25519)
- `SkEcdsaSha2NistP256` (FIDO2 keys using ECDSA P-256)

## Features

- ✅ Interactive terminal UI: see and toggle UP/UV flags as checkboxes
- ✅ Supports both encrypted and unencrypted SSH keys
- ✅ Hidden password input (uses `rpassword` crate) - passphrase won't show on terminal
- ✅ Save with no password, same password, or new password
- ✅ Non-destructive operation - modifies the file in-place
- ✅ UI updates in place (no scrolling)
- ✅ Proper error handling and reporting

## Installation

### Prerequisites
- Rust 1.70 or later

### Build from Source

```bash
git clone https://github.com/junderw/sshkey-fixer.git
cd sshkey-fixer
cargo build --release
```

The compiled binary will be at `target/release/sshkey-fixer`.

## Usage

```bash
sshkey-fixer <path-to-ssh-key-file>
```

### Example

```bash
# Edit an unencrypted key
./sshkey-fixer ~/.ssh/id_ecdsa_sk

# Edit an encrypted key (you'll be prompted for the passphrase)
./sshkey-fixer ~/.ssh/id_ed25519_sk
```

### Interactive Workflow

1. **Load the key**: The tool reads the SSH private key file in OpenSSH format
2. **Check encryption**: If the key is encrypted, you'll be prompted for the passphrase (input is hidden)
3. **Decrypt** (if needed): The key is decrypted using the provided passphrase
4. **Interactive menu**: The UI displays the current state of the UP and UV flags as checkboxes, and presents a menu:
	- Toggle User Presence (UP)
	- Toggle User Verification (UV)
	- Save without password
	- Save with same password (if originally encrypted)
	- Save with new password
	- Quit without saving
5. **UI updates in place**: The screen is cleared and redrawn after each action, so only the current state is visible.
6. **Save**: The modified key is written back to the original file with your chosen password option.


## UI Example

```
SSH Key Flag Editor
===================

File: /home/user/.ssh/id_ed25519_sk
Key type: SkEd25519

Current flags:
	[O] User Presence (UP)
	[ ] User Verification (UV)

1. Toggle User Presence (UP)
2. Toggle User Verification (UV)
3. Save without password
4. Save with same password
5. Save with new password
6. Quit without saving

Toggled User Verification flag.

Enter choice: 2
```

After each action, the UI is redrawn in place, so only the current state is visible.

## Technical Details

### UV_REQUIRED Flag
The UV_REQUIRED flag (0x04) is a single bit that indicates the security key requires user verification. This is important for:
- Ensuring the key cannot be used without physical interaction
- Maintaining security policies that require user verification
- Compatibility with systems that check for this flag

### UP_REQUIRED Flag
The UP_REQUIRED flag (0x01) is a single bit that indicates the security key requires user presence. This is important for:
- Ensuring the key cannot be used without physical interaction (pressing the button)
- Maintaining security policies that require explicit user confirmation
- Preventing accidental or automated unauthorized use

### Implementation Notes

ssh-key dependency has been patched to include the ability to set flag bits.
The git hash is directly written in the dependency in Cargo.toml.

See the diff here:

https://github.com/RustCrypto/SSH/compare/ssh-key/v0.6.7...junderw:RustCrypto-SSH:57ced034d1a87853626695616545ce36d79c515e

## Dependencies

- `ssh-key` (0.6.7) - SSH key parsing and handling with encryption support
- `rpassword` (7.3+) - Hidden password input
- `rand` (0.8) - Random number generation for encryption

## Error Handling

The tool provides clear error messages for common issues:

- **Invalid key file**: "Error: Failed to read key file"
- **Unsupported key type**: "Error: Unsupported key type. This tool only supports SkEd25519 and SkEcdsaSha2NistP256 keys."
- **Wrong passphrase**: "Error: Failed to decrypt private key with provided passphrase"
- **File permission issues**: "Error: Failed to write modified key"

## Security Considerations

- **Password handling**: Passwords are read using `rpassword`, which disables terminal echo to prevent password visibility
- **In-place modification**: The original file is modified directly - ensure you have backups
- **Encryption preservation**: If a key was encrypted, it remains encrypted with the same passphrase after modification

## Building for Distribution

```bash
# Build optimized release binary
cargo build --release

# The binary will be at: target/release/sshkey-fixer
```

## License

MIT

## Troubleshooting

### "The private key is of another type"
This error occurs when using a key type that isn't SkEd25519 or SkEcdsaSha2NistP256. These are the only FIDO2/U2F security key types currently supported.

### "Failed to decrypt private key"
Double-check that you entered the correct passphrase. The error will occur if the passphrase is incorrect.

### File not modified after running
Check if the UV_REQUIRED flag was already set. The tool will skip modification if the flag is already present, but the command will still complete successfully.

## Related Links

- [OpenSSH Key Format](https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.key)
- [FIDO2 Specification](https://fidoalliance.org/fido2/)
