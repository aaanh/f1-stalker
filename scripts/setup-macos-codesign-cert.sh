#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cert_name="${CODESIGN_CERT_NAME:-F1 Stalker Codesign}"
ca_name="${CODESIGN_CA_NAME:-F1 Stalker Local CA}"
cert_dir="$root/target/codesign"
keychain="${KEYCHAIN:-$HOME/Library/Keychains/login.keychain-db}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS only" >&2
  exit 1
fi

if security find-identity -v -p codesigning | awk -F'"' -v name="$cert_name" '$2 == name { found++ } END { exit(found == 0) }'; then
  echo "codesign identity already in keychain: $cert_name"
  security find-identity -v -p codesigning | awk -F'"' -v name="$cert_name" '$2 == name'
  exit 0
fi

mkdir -p "$cert_dir"

openssl genrsa -out "$cert_dir/ca.key" 4096
openssl req -x509 -new -nodes -key "$cert_dir/ca.key" -sha256 -days 3650 \
  -out "$cert_dir/ca.crt" \
  -subj "/CN=${ca_name}/O=F1 Stalker/C=US"

openssl genrsa -out "$cert_dir/key.pem" 4096
openssl req -new -key "$cert_dir/key.pem" -out "$cert_dir/cert.csr" \
  -subj "/CN=${cert_name}/O=F1 Stalker/C=US"

cat >"$cert_dir/cert.ext" <<EOF
basicConstraints=critical,CA:FALSE
keyUsage=critical,digitalSignature
extendedKeyUsage=critical,codeSigning
EOF

openssl x509 -req -in "$cert_dir/cert.csr" -CA "$cert_dir/ca.crt" -CAkey "$cert_dir/ca.key" \
  -CAcreateserial -out "$cert_dir/cert.pem" -days 825 -sha256 -extfile "$cert_dir/cert.ext"

security import "$cert_dir/key.pem" \
  -k "$keychain" \
  -T /usr/bin/codesign \
  -T /usr/bin/security \
  -A

security import "$cert_dir/cert.pem" \
  -k "$keychain" \
  -T /usr/bin/codesign \
  -T /usr/bin/security \
  -A

security import "$cert_dir/ca.crt" \
  -k "$keychain" \
  -T /usr/bin/codesign \
  -T /usr/bin/security \
  -A

if security add-trusted-cert -d -r trustRoot -p codeSign -k "$keychain" "$cert_dir/ca.crt" 2>/dev/null; then
  :
else
  echo "note: trust the local CA in Keychain Access if codesign identity is missing"
fi

if security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "" "$keychain" 2>/dev/null; then
  :
fi

if ! security find-identity -v -p codesigning | grep -Fq "$cert_name"; then
  echo "codesign identity not visible yet; falling back to ad-hoc signing in builds" >&2
  echo "open Keychain Access → login → Certificates → trust \"${ca_name}\" for Code Signing" >&2
  exit 1
fi

echo "installed self-signed codesign identity: $cert_name"
security find-identity -v -p codesigning | grep "$cert_name" || true
echo
echo "Gatekeeper will still show an unidentified developer prompt for downloads."
echo "Users can open once via Right-click → Open, or:"
echo "  xattr -dr com.apple.quarantine \"/Applications/F1 Stalker.app\""
