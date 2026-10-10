#!/usr/bin/env bash
set -euo pipefail

ROOT="${INSTALLER_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
HELPER_SOURCE="${MENDELEY_HELPER_BIN:?Set MENDELEY_HELPER_BIN to compiled Rust helper}"
TEMP_DIR="$(mktemp -d)"
FAKE_BIN="$TEMP_DIR/bin"
PACKAGE_ROOT="$TEMP_DIR/installer"
PLUGIN_GUID="{BE5CBF95-C0AD-4842-B157-AC40FEDD9441}"
HELPER_STARTED=false
mkdir -p "$FAKE_BIN"
trap 'if [[ "$HELPER_STARTED" == true ]]; then "$HELPER_SOURCE" --stop >/dev/null 2>&1 || true; fi; rm -rf "$TEMP_DIR"' EXIT

if curl --fail --silent http://127.0.0.1:8080/health >/dev/null; then
    echo "Mendeley helper already running on port 8080; refusing to stop existing service" >&2
    exit 1
fi

INSTALLER_ARCHIVE="${MENDELEY_INSTALLER_ARCHIVE:-$TEMP_DIR/mendeley-linux-installer.tar.gz}"
if [[ -z "${MENDELEY_INSTALLER_ARCHIVE:-}" ]]; then
    mkdir -p "$TEMP_DIR/package-source/bin/$(uname -m)"
    cp "$ROOT/install.sh" "$TEMP_DIR/package-source/"
    cp "$ROOT/config.json" "$ROOT/index.html" "$ROOT/oauth.html" "$TEMP_DIR/package-source/"
    cp -R "$ROOT/scripts" "$ROOT/resources" "$ROOT/translations" "$ROOT/vendor" "$ROOT/licenses" \
        "$TEMP_DIR/package-source/"
    rm -f "$TEMP_DIR/package-source/scripts/mendeley-loopback-server.py"
    install -m 755 "$HELPER_SOURCE" \
        "$TEMP_DIR/package-source/bin/$(uname -m)/mendeley-loopback-server"
    tar --exclude='__pycache__' --exclude='*.pyc' -czf "$INSTALLER_ARCHIVE" \
        -C "$TEMP_DIR/package-source" install.sh bin config.json index.html oauth.html \
        scripts resources translations vendor licenses
fi
mkdir -p "$PACKAGE_ROOT"
tar -xzf "$INSTALLER_ARCHIVE" -C "$PACKAGE_ROOT"
[[ -x "$PACKAGE_ROOT/bin/$(uname -m)/mendeley-loopback-server" ]]
[[ ! -e "$PACKAGE_ROOT/scripts/mendeley-loopback-server.py" ]]
cat > "$FAKE_BIN/systemctl" <<'EOF'
#!/usr/bin/env bash
exit 1
EOF
chmod +x "$FAKE_BIN/systemctl"

assert_running() {
    local response
    for attempt in {1..50}; do
        response="$(curl --fail --silent http://127.0.0.1:8080/health || true)"
        [[ "$response" == '{"service":"mendeley-loopback","status":"ok"}' ]] && {
            HELPER_STARTED=true
            return
        }
        sleep 0.1
    done
    echo "Rust helper did not start or return healthy status" >&2
    return 1
}

assert_stopped() {
    for attempt in {1..50}; do
        if ! curl --fail --silent http://127.0.0.1:8080/health >/dev/null; then
            HELPER_STARTED=false
            return
        fi
        sleep 0.1
    done
    echo "Rust helper did not stop" >&2
    return 1
}

install_to() {
    local home="$1" data_home="$2" override_dir="$3" expected_dir="$4"
    mkdir -p "$home"
    env HOME="$home" XDG_CONFIG_HOME="$home/.config" XDG_DATA_HOME="$data_home" \
        ONLYOFFICE_PLUGIN_DIR="$override_dir" PATH="$FAKE_BIN:$PATH" \
        bash "$PACKAGE_ROOT/install.sh" >/dev/null
    [[ -x "$home/.local/bin/mendeley-loopback-server" ]]
    [[ -f "$expected_dir/$PLUGIN_GUID/config.json" ]]
    [[ "$(readlink "$expected_dir/mendeley")" == "$PLUGIN_GUID" ]]
    assert_running
}

native_home="$TEMP_DIR/native-home"
native_data="$native_home/.local/share"
native_plugins="$native_data/onlyoffice/desktopeditors/sdkjs-plugins"
mkdir -p "$(dirname "$native_plugins")"
install_to "$native_home" "$native_data" "" "$native_plugins"

flatpak_home="$TEMP_DIR/flatpak-home"
flatpak_plugins="$flatpak_home/.var/app/org.onlyoffice.desktopeditors/data/onlyoffice/desktopeditors/sdkjs-plugins"
mkdir -p "$(dirname "$flatpak_plugins")"
install_to "$flatpak_home" "$flatpak_home/.local/share" "" "$flatpak_plugins"

snap_home="$TEMP_DIR/snap-home"
snap_plugins="$snap_home/snap/onlyoffice-desktopeditors/current/.local/share/onlyoffice/desktopeditors/sdkjs-plugins"
mkdir -p "$(dirname "$snap_plugins")"
install_to "$snap_home" "$snap_home/.local/share" "" "$snap_plugins"

custom_home="$TEMP_DIR/custom-home"
custom_plugins="$custom_home/editor/plugins/sdkjs-plugins"
install_to "$custom_home" "$custom_home/data" "$custom_plugins" "$custom_plugins"

missing_home="$TEMP_DIR/missing-home"
mkdir -p "$missing_home"
if env HOME="$missing_home" XDG_DATA_HOME="$missing_home/data" PATH="$FAKE_BIN:$PATH" \
    bash "$PACKAGE_ROOT/install.sh" >/dev/null 2>&1; then
    echo "installer accepted unknown plugin path" >&2
    exit 1
fi
[[ ! -e "$missing_home/.local/bin/mendeley-loopback-server" ]]

package_root="$TEMP_DIR/deb/opt/mendeley-onlyoffice"
package_home="$TEMP_DIR/package-home"
mkdir -p "$package_root/bin" "$package_root/plugin" "$package_root/packaging/linux" \
    "$package_home/.local/share/onlyoffice/desktopeditors"
cp "$ROOT/packaging/linux/install-user.sh" "$ROOT/packaging/linux/uninstall-user.sh" \
    "$package_root/packaging/linux/"
install -m 755 "$HELPER_SOURCE" "$package_root/bin/mendeley-loopback-server"
cp "$ROOT/config.json" "$package_root/plugin/"
env HOME="$package_home" bash "$package_root/packaging/linux/install-user.sh" <<<"1" >/dev/null
assert_running
env HOME="$package_home" bash "$package_root/packaging/linux/uninstall-user.sh" >/dev/null
assert_stopped

printf '%s\n' "Linux Rust helper installer and lifecycle checks passed"
