#!/usr/bin/env sh

# Set trigger before exiting
trap 'rm -rf ./SSH' EXIT

# Clone the RustCrypto SSH repository and compare specific subdirectories
git clone -q --depth 1 --branch ssh-key/v0.6.7 https://github.com/RustCrypto/SSH.git >/dev/null 2>&1

# Compare the subdirectories
diff -r ./ssh-cipher ./SSH/ssh-cipher
diff -r ./ssh-encoding ./SSH/ssh-encoding
diff -r ./ssh-key ./SSH/ssh-key
