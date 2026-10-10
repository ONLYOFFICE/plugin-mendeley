<div align="center">

# Mendeley for ONLYOFFICE

**Citations and bibliographies from Linux, without switching to Windows.**

[![License: Apache 2.0](https://img.shields.io/github/license/nashirabbash/plugin-mendeley?style=flat-square)](LICENSE)
[![Latest release](https://img.shields.io/github/v/release/nashirabbash/plugin-mendeley?style=flat-square)](https://github.com/nashirabbash/plugin-mendeley/releases)

![Mendeley plugin demo](assets/demo_video_update.gif)

*Find references, insert citations, and build bibliographies in ONLYOFFICE.*

</div>

## Why I built it

Ever found yourself switching from Linux to Windows and back just to add a Mendeley citation while writing a paper, report, or thesis? That gets tiring fast.

This plugin brings a Mendeley Cite-style workflow to ONLYOFFICE on Linux, so you can keep writing without switching operating systems. It is a fork of [ONLYOFFICE/plugin-mendeley](https://github.com/ONLYOFFICE/plugin-mendeley), adapted and extended for this use. Contributions and forks are welcome.

## What you can do

| Find references | Cite and format | Keep documents editable |
| --- | --- | --- |
| Search by title, author, or year. Browse collections, favorites, and recently added references. | Insert in-text citations and bibliographies. Choose styles such as APA, Chicago, and Harvard, plus citation language. | Edit existing citations or unlink them to convert citations to plain text. |

![Install the plugin, connect a Mendeley account, and add references in ONLYOFFICE](docs/images/mendeley-workflow.png)

## Get started

### Requirements

- ONLYOFFICE Desktop Editors (Flatpak, Snap, or native) or a self-hosted ONLYOFFICE Document Server.
- [Mendeley Reference Manager](https://www.mendeley.com/reference-management/mendeley-reference-manager) installed and signed in (for automatic zero-config connect).
- *(Optional fallback)*: A Mendeley developer OAuth application ID if connecting manually via web OAuth.

### ONLYOFFICE Desktop Editors

Install ONLYOFFICE Desktop Editors first. You can install the Mendeley plugin and Rust loopback helper using any of the methods below:

#### Linux

**One-step Linux installer**

Download `mendeley-linux-installer.tar.gz` from [Releases](https://github.com/nashirabbash/plugin-mendeley/releases), extract it, and run:

```bash
tar -xzf mendeley-linux-installer.tar.gz
./install.sh
```

The installer installs both plugin and Rust helper, with binaries for x86_64 and ARM64 Linux. It detects native, Flatpak, and Snap ONLYOFFICE plugin directories. If installation uses a custom path, set `ONLYOFFICE_PLUGIN_DIR` to its `sdkjs-plugins` directory or enter the path when prompted.

#### Windows

Download `Mendeley-ONLYOFFICE-Setup-x64.exe` for x64 Windows or `Mendeley-ONLYOFFICE-Setup-arm64.exe` for ARM64 Windows from [Releases](https://github.com/nashirabbash/plugin-mendeley/releases), then run the installer. It installs the plugin and Rust helper, starts the helper, and configures it to start at sign-in. Uninstalling removes the startup entry and stops the helper.

#### Manual plugin-only installation (`.plugin`)

The `.plugin` file installs only the ONLYOFFICE plugin. Desktop Editors still need the local helper; use the platform installer above for a complete installation.

1. Download `mendeley.plugin` from [Releases](https://github.com/nashirabbash/plugin-mendeley/releases).
2. Open ONLYOFFICE Desktop Editors.
3. Navigate to **Plugins** -> **Settings** / **Plugin Manager** -> select `mendeley.plugin`.

### ONLYOFFICE Document Server

For a self-hosted Document Server, copy the plugin into its `sdkjs-plugins` directory. Example for a typical Linux installation:

```bash
sudo cp -r plugin-mendeley /var/www/onlyoffice/documentserver/sdkjs-plugins/mendeley
```

Adjust source and destination paths for your checkout and installation. If your server requires a plugin URL or configuration, add the plugin through its integration settings. Plugin GUID: `asc.{BE5CBF95-C0AD-4842-B157-AC40FEDD9441}`.

## Connect your Mendeley account

### 1. Desktop session first

1. Open **Mendeley** from ONLYOFFICE's **Plugins** tab.
2. If Mendeley Reference Manager is running and signed in, the plugin loads your library from its local session.
3. If no desktop session is available, the plugin starts web sign-in automatically. Keep the local helper running for Desktop Editors; it receives the OAuth redirect.

### 2. Web OAuth configuration

To use your own Mendeley developer OAuth application:
1. In the plugin's **Config** screen, enter your Mendeley Application ID.
2. Set the redirect URI in your Mendeley Developer portal:
   - **Desktop Editors:** `http://localhost:8080/`
   - **Document Server:** the HTTPS URL of the plugin's `oauth.html` page on your server.
3. Click **Save**, then click **Login to Mendeley (Web)** to authorize via browser.

![Mendeley OAuth sequence diagram](docs/mendeley-auth-sequence.png)

[View the standalone OAuth flow diagram](docs/mendeley-auth-sequence.html).

### Token behavior

The plugin reuses its browser-stored `mendToken` at startup. If none exists, it checks `127.0.0.1:8080/token` for a saved helper token or a signed-in Mendeley Reference Manager session. If neither provides a token, web OAuth starts automatically. In Desktop Editors, the local helper must be running to prepare and receive web sign-in.

- **Desktop Editors:** Mendeley redirects to the loopback helper, which saves the token. The plugin polls `/token` and stores its own copy in browser storage.
- **Document Server:** Mendeley redirects to the plugin's `oauth.html`, which passes the token to the plugin window. The plugin stores it in browser storage.
- **API and session:** The plugin sends the access token as a Bearer token. It does not refresh tokens automatically. A `401` clears the plugin's browser-stored token and returns to sign-in.

Logging out clears the plugin's browser-stored token. In Desktop Editors, the helper's saved token remains, so the plugin can restore the session when reopened. Starting a new manual login clears both plugin and helper token copies first.

## Use the plugin

1. Place the document cursor where you want the citation.
2. Find references by searching or browsing collections, **Favorites**, or **Recently Added**.
3. Select one or more references.
4. Choose a citation style and language.
5. Select **Insert citation** to add an in-text citation.
6. Select **Insert bibliography** to add the reference list at the end of the document.
7. Select the pencil icon beside a citation to edit it. Select **Unlink all** to convert citations to plain text.

## Troubleshooting

On CentOS with SELinux enabled, restore the security context after installation and restart the Document Server document service:

```bash
sudo restorecon -Rv /var/www/onlyoffice/documentserver/sdkjs-plugins/
sudo supervisorctl restart ds:docservice
```

If Mendeley login reports that port `127.0.0.1:8080` is in use, another application is using the port required by the local helper.

## Contributing

Bug reports and contributions are welcome through the [issue tracker](https://github.com/nashirabbash/plugin-mendeley/issues). This project is distributed under the [Apache License 2.0](LICENSE).
