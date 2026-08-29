#!/usr/bin/env bash
set -euo pipefail

APP_NAME="prism-discord-rpc"
REPO="Lunyyx/prism-discord-rpc"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_PATH="${INSTALL_DIR}/${APP_NAME}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux)
        case "${ARCH}" in
            x86_64)
                ASSET_SUFFIX="linux-x64"
                ;;
            *)
                echo "Error: Unsupported architecture: ${ARCH}"
                exit 1
                ;;
        esac
        ;;

    Darwin)
        case "${ARCH}" in
            x86_64)
                ASSET_SUFFIX="darwin-x64"
                ;;
            arm64)
                ASSET_SUFFIX="darwin-arm64"
                ;;
            *)
                echo "Error: Unsupported architecture: ${ARCH}"
                exit 1
                ;;
        esac
        ;;

    *)
        echo "Error: Unsupported operating system: ${OS}"
        exit 1
        ;;
esac

echo "Installing ${APP_NAME} for ${OS} ${ARCH}..."

VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name":' \
    | sed -E 's/.*"([^"]+)".*/\1/')

if [[ -z "${VERSION}" ]]; then
    echo "Error: Could not determine the latest version."
    exit 1
fi

ASSET="prism-rpc-v${VERSION}-${ASSET_SUFFIX}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"

echo "Version: ${VERSION}"
echo "Asset:   ${ASSET}"

mkdir -p "${INSTALL_DIR}"

TMP_FILE="$(mktemp)"
trap 'rm -f "${TMP_FILE}"' EXIT

echo "Downloading..."
curl -fL "${DOWNLOAD_URL}" -o "${TMP_FILE}"

install -m 755 "${TMP_FILE}" "${BINARY_PATH}"

if [[ "${OS}" == "Linux" ]]; then

    SYSTEMD_DIR="${HOME}/.config/systemd/user"
    SERVICE_PATH="${SYSTEMD_DIR}/${APP_NAME}.service"

    mkdir -p "${SYSTEMD_DIR}"

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

elif [[ "${OS}" == "Darwin" ]]; then

    LAUNCH_AGENTS_DIR="${HOME}/Library/LaunchAgents"
    PLIST_PATH="${LAUNCH_AGENTS_DIR}/${APP_NAME}.plist"

    mkdir -p "${LAUNCH_AGENTS_DIR}"

    cat > "${PLIST_PATH}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${APP_NAME}</string>

    <key>ProgramArguments</key>
    <array>
        <string>${BINARY_PATH}</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>${HOME}/Library/Logs/${APP_NAME}.log</string>

    <key>StandardErrorPath</key>
    <string>${HOME}/Library/Logs/${APP_NAME}.error.log</string>
</dict>
</plist>
EOF

    mkdir -p "${HOME}/Library/Logs"

    # Reload the LaunchAgent if it was already installed
    launchctl bootout "gui/$(id -u)" "${PLIST_PATH}" 2>/dev/null || true
    launchctl bootstrap "gui/$(id -u)" "${PLIST_PATH}"

    echo
    echo "${APP_NAME} has been installed successfully."
    echo "Binary: ${BINARY_PATH}"
    echo "LaunchAgent: ${PLIST_PATH}"

    if launchctl print "gui/$(id -u)/${APP_NAME}" >/dev/null 2>&1; then
        echo "LaunchAgent is running."
    else
        echo "Warning: The LaunchAgent was installed but is not currently running."
        echo "Check logs with:"
        echo "  tail -f ~/Library/Logs/${APP_NAME}.error.log"
    fi

fi