#!/usr/bin/env bash
set -euo pipefail

readonly APP_NAME="Penna"
readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly BUILT_APP="${PROJECT_ROOT}/src-tauri/target/release/bundle/macos/${APP_NAME}.app"
readonly INSTALLED_APP="/Applications/${APP_NAME}.app"

fail() {
  echo "error: $1" >&2
  exit 1
}

require_built_app() {
  [[ -d "${BUILT_APP}" ]] || fail "${BUILT_APP} not found, run 'make dmg' first"
}

quit_running_app() {
  if pgrep -x "${APP_NAME}" > /dev/null || pgrep -x "penna" > /dev/null; then
    pkill -x "${APP_NAME}" || pkill -x "penna" || true
    sleep 1
  fi
}

copy_to_applications() {
  rm -rf "${INSTALLED_APP}"
  ditto "${BUILT_APP}" "${INSTALLED_APP}"
}

main() {
  require_built_app
  quit_running_app
  copy_to_applications
  echo "${APP_NAME} installed in ${INSTALLED_APP}"
}

main "$@"
