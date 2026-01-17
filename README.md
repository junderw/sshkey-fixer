# SSH Key Fixer

A command-line utility written in Rust that modifies SSH security key files by setting the UV (User Verification) REQUIRED flag for OpenSSH FIDO2/U2F keys.

## Overview

There was an issue in openssh when using `ssh-keygen -K` to output the ssh key files for security keys. It turns out that UV REQ flag was not being properly output into the key file when the security key has internal user verification (like Yubikey BIO).

https://github.com/Yubico/libfido2/discussions/926#discussioncomment-15523431

This discussion explains the issue. This tool will fix an affected key file by setting the UV REQ flag properly.

**Supported key types:**
- `SkEd25519` (FIDO2 keys using Ed25519)
- `SkEcdsaSha2NistP256` (FIDO2 keys using ECDSA P-256)

## Features

- ✅ Supports both encrypted and unencrypted SSH keys
- ✅ Hidden password input (uses `rpassword` crate) - passphrase won't show on terminal
- ✅ Automatic re-encryption with original password if key was encrypted
- ✅ Non-destructive operation - modifies the file in-place
- ✅ Checks if flag is already set - skips modification if unnecessary
- ✅ Clear status messages about key type and modifications
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
# Fix an unencrypted key
./sshkey-fixer ~/.ssh/id_ecdsa_sk

# Fix an encrypted key (you'll be prompted for the passphrase)
./sshkey-fixer ~/.ssh/id_ed25519_sk
```

### Workflow

1. **Load the key**: The tool reads the SSH private key file in OpenSSH format
2. **Check encryption**: If the key is encrypted, you'll be prompted for the passphrase (input is hidden)
3. **Decrypt** (if needed): The key is decrypted using the provided passphrase
4. **Modify flag**: The UV_REQUIRED flag is set if not already present
5. **Re-encrypt** (if needed): If the key was originally encrypted, it's re-encrypted with the same passphrase
6. **Save**: The modified key is written back to the original file

## Output Examples

### For Unencrypted Keys
```
The private key is not encrypted.
Key type: SkEd25519
Current flags: 0x21
New flags: 0x25
SSH key successfully modified.
```

### For Encrypted Keys
```
The private key is encrypted.
Enter passphrase: 
Key type: SkEd25519
Current flags: 0x21
New flags: 0x25
SSH key successfully modified.
```

### When Flag is Already Set
```
The private key is not encrypted.
Key type: SkEd25519
Current flags: 0x25
UV_REQUIRED flag is already set. No changes needed.
SSH key successfully modified.
```

## Technical Details

### UV_REQUIRED Flag
The UV_REQUIRED flag (0x04) is a single bit that indicates the security key requires user verification. This is important for:
- Ensuring the key cannot be used without physical interaction
- Maintaining security policies that require user verification
- Compatibility with systems that check for this flag

### Implementation Notes

This tool vendors the ssh-key crate and patches it to give us the ability to set the flag byte.

## Dependencies

- `ssh-key` (0.6.7) - SSH key parsing and handling with encryption support
- `rpassword` (7.3+) - Hidden password input
- `rand` (0.8) - Random number generation for encryption
- `base64` (0.22.1) - Base64 encoding/decoding
- `regex` (1.12.2) - Regular expressions (for potential future use)

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
