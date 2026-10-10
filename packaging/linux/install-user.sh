#!/usr/bin/env bash
set -euo pipefail

SCRIPT_PATH="$(readlink -f "$0")"
PACKAGE_ROOT="$(cd "$(dirname "$SCRIPT_PATH")/../.." && pwd)"
PLUGIN_SOURCE="${PACKAGE_ROOT}/plugin"
HELPER="${PACKAGE_ROOT}/bin/mendeley-loopback-server"
PLUGIN_GUID="{BE5CBF95-C0AD-4842-B157-AC40FEDD9441}"
PLUGIN_NAME="mendeley"

if [[ ! -x "$HELPER" || ! -d "$PLUGIN_SOURCE" ]]; then
    printf 'Mendeley package payload missing. Reinstall package.\n' >&2
    exit 1
fi

mapfile -t CANDIDATES < <(
    for root in \
        "$HOME/.local/share/onlyoffice/desktopeditors" \
        "$HOME/.var/app/org.onlyoffice.desktopeditors/data/onlyoffice/desktopeditors"; do
        [[ -d "$root" ]] && printf '%s/sdkjs-plugins\n' "$root"
    done
)

if ((${#CANDIDATES[@]} == 0)); then
    printf 'No ONLYOFFICE plugin directory found. Install ONLYOFFICE Desktop Editors first.\n' >&2
    exit 1
fi

printf 'Choose ONLYOFFICE plugin directory:\n'
for index in "${!CANDIDATES[@]}"; do
    printf '  %d) %s\n' "$((index + 1))" "${CANDIDATES[$index]}"
done
read -r -p 'Selection: ' selection
if [[ ! "$selection" =~ ^[0-9]+$ ]] || ((selection < 1 || selection > ${#CANDIDATES[@]})); then
    printf 'Invalid selection.\n' >&2
    exit 1
fi

plugins_parent="${CANDIDATES[$((selection - 1))]}"
plugin_dir="${plugins_parent}/${PLUGIN_GUID}"
temp_dir="${plugin_dir}.new.$$"
backup_dir="${plugin_dir}.backup.$$"
mkdir -p "$plugins_parent"
cp -R "$PLUGIN_SOURCE" "$temp_dir"
if [[ -e "$plugin_dir" ]]; then
    mv "$plugin_dir" "$backup_dir"
fi
if ! mv "$temp_dir" "$plugin_dir"; then
    [[ ! -e "$backup_dir" ]] || mv "$backup_dir" "$plugin_dir"
    exit 1
fi
rm -rf "$backup_dir"
ln -sfn "$PLUGIN_GUID" "${plugins_parent}/${PLUGIN_NAME}" 2>/dev/null || true

CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
CONFIG_DIR="$CONFIG_HOME/mendeley-onlyoffice"
AUTOSTART_DIR="$CONFIG_HOME/autostart"
mkdir -p "$CONFIG_DIR" "$AUTOSTART_DIR"
printf '%s\n' "$plugin_dir" > "$CONFIG_DIR/install-path"
chmod 600 "$CONFIG_DIR/install-path"
cat > "$AUTOSTART_DIR/mendeley-loopback.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Mendeley ONLYOFFICE Helper
Exec=$HELPER
X-GNOME-Autostart-enabled=true
NoDisplay=true
EOF
chmod 600 "$AUTOSTART_DIR/mendeley-loopback.desktop"
"$HELPER" --stop >/dev/null 2>&1 || true
"$HELPER" >/dev/null 2>&1 &
printf 'Installed Mendeley plugin at %s. Rust helper started and configured for login.\n' "$plugin_dir"
