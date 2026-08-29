#!/usr/bin/env bash
set -euo pipefail

APP_NAME="prism-discord-rpc"
REPO="Lunyyx/prism-discord-rpc"
INSTALL_DIR="${HOME}/.local/bin"
SYSTEMD_DIR="${HOME}/.config/systemd/user"
BINARY_PATH="${INSTALL_DIR}/${APP_NAME}"
SERVICE_PATH="${SYSTEMD_DIR}/${APP_NAME}.service"

echo "Installing ${APP_NAME}..."

if [[ "$(uname -s)" != "Linux" ]]; then
    echo "Error: ${APP_NAME} currently only supports Linux."
    exit 1
fi

case "$(uname -m)" in
    x86_64)
        ASSET_SUFFIX="linux-x64"
        ;;
    *)
        echo "Error: Unsupported architecture: $(uname -m)"
        exit 1
        ;;
esac

VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name":' \
    | sed -E 's/.*"([^"]+)".*/\1/')

ASSET="prism-rpc-v${VERSION}-${ASSET_SUFFIX}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"

if [[ -z "${DOWNLOAD_URL}" ]]; then
    echo "Error: Could not find ${ASSET} in the latest GitHub release."
    exit 1
fi

mkdir -p "${INSTALL_DIR}" "${SYSTEMD_DIR}"

TMP_FILE="$(mktemp)"
trap 'rm -f "${TMP_FILE}"' EXIT

curl -fL "${DOWNLOAD_URL}" -o "${TMP_FILE}"
install -m 755 "${TMP_FILE}" "${BINARY_PATH}"

cat > "${SERVICE_PATH}" <<EOF
[Unit]
Description=Prism Discord RPC
After=graphical-session.target

[Service]
ExecStart=${BINARY_PATH}
Restart=on-failure

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now "${APP_NAME}.service"

echo
echo "${APP_NAME} has been installed successfully."
echo "Binary:  ${BINARY_PATH}"
echo "Service: ${SERVICE_PATH}"

if systemctl --user is-active --quiet "${APP_NAME}.service"; then
    echo "Service is running."
else
    echo "Warning: The service was installed but is not currently running."
    echo "Check logs with:"
    echo "  journalctl --user -u ${APP_NAME}.service"
fi
