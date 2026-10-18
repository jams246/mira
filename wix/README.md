# Windows installer

Install `cargo-wix` 0.3.9 (`cargo install cargo-wix --version 0.3.9 --locked`)
and the WiX Toolset 3.14.1 build tools. Add the directory containing
`candle.exe` and `light.exe` to `PATH`, or set `WIX` to the installed WiX root
directory (which contains `bin`). Build from a Visual Studio developer shell
with the Windows SDK installed. If GPUI cannot find `fxc.exe`, set
`GPUI_FXC_PATH` to its full path in the Windows SDK `bin/<version>/x64` folder.

Run `make build:msi` from the repository root. This builds the optimized,
statically linked x64 application using `.cargo/release.toml`, then packages
it with `cargo wix`. The installer is written to
`target/wix/mira-<version>-x86_64.msi`.

Without Make, run:

```powershell
cargo build --release --locked --bin mira --target x86_64-pc-windows-msvc --config .cargo/release.toml
cargo wix --no-build --target x86_64-pc-windows-msvc
```

The MSI installs for the current user in `%LOCALAPPDATA%\MIRA`, creates a
Start menu shortcut, and supports removal through Windows Installed apps.
It includes the license and uses the existing application icon. It does not
require administrator access, change `PATH`, or take over file associations.

Increase the package version in `Cargo.toml` for each installer release.
Keep the `UpgradeCode` and component GUID in `main.wxs` stable so upgrades
replace earlier installations. Downgrades are blocked. Installers are unsigned
unless signed separately with a code-signing certificate.
Custom installation paths are retained for upgrades and uninstall.
