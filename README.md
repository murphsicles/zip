# Zip Wallet 🚀

High-performance, non-custodial BSV wallet with swipe-to-pay, PayMail, and hybrid auth (OAuth 2.1 + Passkeys). Built in Rust for efficiency and scalability. 💪

## Features 📋
- Non-custodial BSV wallet with rust-sv integration 🔑
- PayMail resolution and P2P transactions via paymail-rs 📧
- Advanced scripting with nPrint 📜
- Real-time indexing with RustBus 🔍
- Cross-platform UI with Dioxus v0.6 (desktop/mobile) 📱
- Embeddable swipe button for external projects 🛠️
- Pre-created UTXOs for rapid payments ⚡
- Secure storage with Sled and OS keyring 🛡️

## Installation 🛠️
```shell
git clone https://github.com/murphsicles/zip
cd zip
cargo build --release
```

## Usage ▶️
Set environment variables in `.env` (e.g., ZIP_OAUTH_CLIENT_ID). 🔧
Run the app:
```shell
cargo run
```

## Testing 🧪
```shell
cargo test
```

## Benchmarks 📈
```shell
cargo bench
```

## License 📄
Open BSV
