#!/usr/bin/env bash
# Creates the minisign key pair that signs release checksums.
#
# 1. Run this once, on a machine you trust. It writes the secret key to
#    $OUT_DIR (default ./update-signing, git-ignored) and prints the public key.
# 2. Store the secret key file's contents as the MINISIGN_SECRET_KEY repository
#    secret and its password as MINISIGN_PASSWORD.
# 3. Paste the public key into UPDATE_PUBLIC_KEY in
#    crates/broquest/src/update_manager.rs and release a new version.
#    Builds from before that release cannot verify signatures, so keep the
#    unsigned update path working for one release.
set -euo pipefail

if ! command -v minisign >/dev/null 2>&1; then
    echo "minisign is not installed (apt install minisign / brew install minisign)" >&2
    exit 1
fi

OUT_DIR="${OUT_DIR:-./update-signing}"
mkdir -p "$OUT_DIR"
minisign -G -p "$OUT_DIR/minisign.pub" -s "$OUT_DIR/minisign.key"

echo
echo "Public key (for UPDATE_PUBLIC_KEY):"
tail -n 1 "$OUT_DIR/minisign.pub"
echo
echo "Secret key written to $OUT_DIR/minisign.key; store it as the MINISIGN_SECRET_KEY secret."
