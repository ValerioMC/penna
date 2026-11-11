#!/usr/bin/env bash
set -euo pipefail

readonly CERTIFICATE_NAME="${1:-Penna Local Signing}"
readonly KEYCHAIN="${HOME}/Library/Keychains/login.keychain-db"
readonly TEMPORARY_PASSWORD="penna-temporary"
WORK_DIR="$(mktemp -d)"
readonly WORK_DIR
trap 'rm -rf "${WORK_DIR}"' EXIT

identity_exists() {
  security find-identity -v -p codesigning | grep -q "\"${CERTIFICATE_NAME}\""
}

remove_untrusted_leftover() {
  if security find-certificate -c "${CERTIFICATE_NAME}" "${KEYCHAIN}" > /dev/null 2>&1; then
    security delete-identity -c "${CERTIFICATE_NAME}" "${KEYCHAIN}"
  fi
}

write_certificate_request() {
  cat > "${WORK_DIR}/request.cnf" <<CONFIG
[req]
distinguished_name = subject
x509_extensions = extensions
prompt = no
[subject]
CN = ${CERTIFICATE_NAME}
[extensions]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
CONFIG
}

generate_certificate() {
  write_certificate_request
  openssl req -x509 -newkey rsa:2048 -nodes -days 3650 \
    -config "${WORK_DIR}/request.cnf" \
    -keyout "${WORK_DIR}/key.pem" -out "${WORK_DIR}/certificate.pem" 2> /dev/null
  openssl pkcs12 -export \
    -inkey "${WORK_DIR}/key.pem" -in "${WORK_DIR}/certificate.pem" \
    -out "${WORK_DIR}/certificate.p12" -passout "pass:${TEMPORARY_PASSWORD}"
}

import_into_keychain() {
  security import "${WORK_DIR}/certificate.p12" -k "${KEYCHAIN}" \
    -P "${TEMPORARY_PASSWORD}" -T /usr/bin/codesign
}

trust_for_code_signing() {
  security add-trusted-cert -r trustRoot -p codeSign -k "${KEYCHAIN}" "${WORK_DIR}/certificate.pem"
}

main() {
  if identity_exists; then
    echo "'${CERTIFICATE_NAME}' already exists, nothing to do"
    return
  fi
  remove_untrusted_leftover
  generate_certificate
  import_into_keychain
  trust_for_code_signing
  identity_exists || { echo "error: '${CERTIFICATE_NAME}' is not a valid signing identity" >&2; exit 1; }
  echo "created the signing identity '${CERTIFICATE_NAME}'"
}

main "$@"
