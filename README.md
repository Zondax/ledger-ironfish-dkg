# Ledger Ironfish DKG
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![GithubActions](https://github.com/zondax/ledger-ironfish-dkg/actions/workflows/main.yml/badge.svg)](https://github.com/Zondax/ledger-ironfish-dkg/blob/master/.github/workflows/main.yaml)

---

![zondax_light](docs/zondax_light.png#gh-light-mode-only)
![zondax_dark](docs/zondax_dark.png#gh-dark-mode-only)

_Please visit our website at [zondax.ch](https://www.zondax.ch)_

---

## Compilation and load

You can follow the following steps to setup a development environment on a host running a Debian based Linux distribution (such as Ubuntu).

### Prerequisites

* Install the [Rust language](https://www.rust-lang.org/)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

* Install Ledger Rust building tools dependencies

```bash
# Clang compiler, GCC ARM cross-compiling toolchain 
apt install clang gcc-arm-none-eabi gcc-multilib
# Rust nightly toolchain used to compile ledger devices binaries
rustup install nightly-2023-11-10
# Setup the custom nightly Rust toolchain as default
rustup default nightly-2023-11-10
# Install required component of the nightly toolchain
rustup component add rust-src --toolchain nightly-2023-11-10
```

* Install [ledgerwallet](https://github.com/LedgerHQ/ledgerctl/) and [cargo-ledger](https://github.com/LedgerHQ/cargo-ledger)

```bash
# Install ledgerwallet, a Python dependency of cargo-ledger to sideload binaries on Ledger devices
pip install ledgerwallet
# Install latest cargo-ledger from crates.io
cargo install cargo-ledger
# Run cargo-ledger command to install custom target files on the custom nightly toolchain
cargo ledger setup
```

You are now ready to build the app for Ledger devices!

### Building

Now that you have followed the [prerequisites](#prerequisites) guide, you can build the app with the following command executed in the root directory of the app.

```bash
cargo ledger build nanox
```

This command will build the app for the Nano X, but you can use any supported device (`nanox`, `nanosplus`, `stax`, `flex`)