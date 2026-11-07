# Installation

## Packages

Each [release](https://github.com/Benji377/tooka/releases) provides:

| System | Files |
|---|---|
| Windows | `.msi` installer, also available through winget: `winget install Benji377.Tooka` |
| macOS | `.dmg` (Apple Silicon and Intel) |
| Linux | `.deb`, `.rpm` and `.AppImage` (x86_64); `.deb` and `.rpm` for ARM (aarch64, armv7) |

## With Cargo

If you have [Rust](https://rustup.rs) installed:

```bash
cargo install tooka
```

## From source

Requires Rust 1.89 or later.

```bash
git clone https://github.com/Benji377/tooka.git
cd tooka
cargo build --release
# the program is now at target/release/tooka
```

## Shell completions

```bash
tooka completions bash > ~/.local/share/bash-completion/completions/tooka
tooka completions zsh  > ~/.zfunc/_tooka          # with ~/.zfunc in $fpath
tooka completions fish > ~/.config/fish/completions/tooka.fish
```

PowerShell and Elvish are supported as well (`tooka completions powershell`,
`tooka completions elvish`).

Next: [Getting started](getting-started.md).
