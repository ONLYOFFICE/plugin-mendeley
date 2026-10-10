#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_GUID="{BE5CBF95-C0AD-4842-B157-AC40FEDD9441}"

echo "=========================================================="
echo "  Mendeley ONLYOFFICE Auto-Connect Installer (Linux)      "
echo "=========================================================="
TARGETS=()
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
if [[ -n "${ONLYOFFICE_PLUGIN_DIR:-}" ]]; then
    TARGETS+=("$ONLYOFFICE_PLUGIN_DIR")
else
    POSSIBLE_TARGETS=(
        "$DATA_HOME/onlyoffice/desktopeditors/sdkjs-plugins"
        "$HOME/.var/app/org.onlyoffice.desktopeditors/data/onlyoffice/desktopeditors/sdkjs-plugins"
        "$HOME/snap/onlyoffice-desktopeditors/current/.local/share/onlyoffice/desktopeditors/sdkjs-plugins"
    )
    for target in "${POSSIBLE_TARGETS[@]}"; do
        if [[ -d "$(dirname "$target")" ]]; then
            TARGETS+=("$target")
        fi
    done
fi

if ((${#TARGETS[@]} == 0)); then
    if [[ -t 0 ]]; then
        read -r -p "ONLYOFFICE sdkjs-plugins path not found. Enter path: " target
        [[ -n "$target" ]] || { echo "No plugin path supplied." >&2; exit 1; }
        TARGETS+=("$target")
    else
        echo "ONLYOFFICE plugin path not found. Set ONLYOFFICE_PLUGIN_DIR to its sdkjs-plugins directory and rerun." >&2
        exit 1
    fi
fi

for target in "${TARGETS[@]}"; do
    if [[ "$target" != /*/sdkjs-plugins ]]; then
        echo "Plugin path must be an absolute sdkjs-plugins directory: $target" >&2
        exit 1
    fi
done

HELPER_ARCH="$(uname -m)"
case "$HELPER_ARCH" in
    x86_64|aarch64) ;;
    *)
        echo "Unsupported Linux architecture for Mendeley helper: $HELPER_ARCH" >&2
        exit 1
        ;;
esac
HELPER_SOURCE="$SCRIPT_DIR/bin/$HELPER_ARCH/mendeley-loopback-server"
if [[ ! -x "$HELPER_SOURCE" ]]; then
    echo "Mendeley helper binary missing for $HELPER_ARCH. Re-download the installer." >&2
    exit 1
fi
mkdir -p "$HOME/.local/bin"
cp -f "$HELPER_SOURCE" "$HOME/.local/bin/mendeley-loopback-server"
chmod +x "$HOME/.local/bin/mendeley-loopback-server"
echo "✓ Installed Rust helper at: $HOME/.local/bin/mendeley-loopback-server"

# 2. Configure systemd user service & autostart desktop entry
mkdir -p "$HOME/.config/systemd/user" "$HOME/.config/autostart"
cat > "$HOME/.config/systemd/user/mendeley-loopback.service" <<EOF
[Unit]
Description=Mendeley ONLYOFFICE Loopback Helper
After=network.target

[Service]
Type=simple
ExecStart=$HOME/.local/bin/mendeley-loopback-server
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF

cat > "$HOME/.config/autostart/mendeley-loopback.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Mendeley ONLYOFFICE Helper
Exec=$HOME/.local/bin/mendeley-loopback-server
X-GNOME-Autostart-enabled=true
NoDisplay=true
EOF
chmod 600 "$HOME/.config/autostart/mendeley-loopback.desktop"

"$HOME/.local/bin/mendeley-loopback-server" --stop >/dev/null 2>&1 || true
HELPER_STARTED=false
if command -v systemctl >/dev/null 2>&1 && systemctl --user daemon-reload 2>/dev/null; then
    systemctl --user enable mendeley-loopback.service 2>/dev/null || true
    if systemctl --user restart mendeley-loopback.service 2>/dev/null ||
       systemctl --user start mendeley-loopback.service 2>/dev/null; then
        HELPER_STARTED=true
    fi
fi

if [[ "$HELPER_STARTED" != true ]]; then
    "$HOME/.local/bin/mendeley-loopback-server" --stop >/dev/null 2>&1 || true
    "$HOME/.local/bin/mendeley-loopback-server" >/dev/null 2>&1 &
fi
echo "✓ Helper service running in background (port 8080)"

for target in "${TARGETS[@]}"; do

    mkdir -p "$target/$PLUGIN_GUID"
    cp -rf "$SCRIPT_DIR/config.json" "$SCRIPT_DIR/index.html" "$SCRIPT_DIR/oauth.html" \
           "$SCRIPT_DIR/scripts" "$SCRIPT_DIR/resources" "$SCRIPT_DIR/translations" \
           "$SCRIPT_DIR/vendor" "$SCRIPT_DIR/licenses" \
           "$target/$PLUGIN_GUID/"
    ln -sfn "$PLUGIN_GUID" "$target/mendeley" 2>/dev/null || true
    echo "✓ Plugin installed at: $target/$PLUGIN_GUID"
done

echo "=========================================================="
echo "  Installation completed successfully!                    "
echo "  Launch ONLYOFFICE Desktop Editors and Mendeley will      "
echo "  connect automatically.                                  "
echo "=========================================================="
