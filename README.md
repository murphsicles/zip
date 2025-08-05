# Zip Wallet 🚀

High-performance, non-custodial, cross-platform BSV wallet with embedible swipe-to-pay buttons, PayMail, and hybrid authentication (OAuth 2.1 + Passkeys). Built in Rust for efficiency, scalability, and security. 💪

## Features 📋
- **Non-Custodial BSV Wallet**: Secure HD wallet with `rust-sv` for key generation and address derivation. 🔑
- **PayMail Integration**: Supports alias creation, resolution, and P2P transactions via `paymail-rs`. 📧
- **Hybrid Authentication**: OAuth 2.1 (Google) and Passkey support with 2FA (TOTP) for secure login. 🔒
- **Real-Time Indexing**: RustBus integration for balance and transaction history queries. 🔍
- **Cross-Platform UI**: Built with Dioxus v0.6 for desktop and mobile, featuring responsive components (`Home`, `Auth`, `Profile`, `Settings`, etc.). 📱
- **Dynamic Theming**: Light/dark theme support with user-configurable settings. 🌗
- **Swipe-to-Pay**: Embeddable `SwipeButton` for rapid payments in external projects. ⚡
- **Pre-Created UTXOs**: Optimized for fast transaction building. 🚀
- **Secure Storage**: Sled database and OS keyring for private keys and user data. 🛡️
- **Telemetry & Monitoring**: Tracks auth and payment events with local logging and optional external endpoint reporting. 📊
- **Rate Limiting**: Protects auth and payment operations (5 requests/minute). 🛑
- **Input Validation**: Sanitizes emails, validates PayMail prefixes, TOTP codes, and currency inputs. ✅
- **Caching**: Efficient price caching with TTL for balance conversions. ⏱️
- **Error Handling**: Unified `ZipError` with user-friendly messages via `ErrorDisplay`. ❗
- **Session Management**: Robust session tracking with timestamps and telemetry. 🔐

## Installation 🛠️
```shell
git clone https://github.com/murphsicles/zip
cd zip
cargo build --release
```

## Usage ▶️
1. Set environment variables in `.env`:
   - `OAUTH_CLIENT_ID`, `OAUTH_CLIENT_SECRET`, `OAUTH_AUTH_URL`, `OAUTH_TOKEN_URL`, `OAUTH_REDIRECT_URI` for OAuth.
   - `RUSTBUS_ENDPOINT` for RustBus integration.
   - `LOG_LEVEL` (e.g., `info`, `debug`) for logging.
   - `TELEMETRY_ENDPOINT` for optional external telemetry reporting.
2. Run the app:
```shell
cargo run
```

## Testing 🧪
Run all tests to verify functionality:
```shell
cargo test
```
- **Auth Tests**: Covers OAuth, Passkey, 2FA, session management, and email validation (`tests/auth_tests.rs`).
- **Blockchain Tests**: Validates wallet address generation, balance updates, PayMail aliases, and rate limiting (`tests/blockchain_tests.rs`).
- **UI Tests**: Ensures rendering of `Home`, `Auth`, `Profile`, `Settings`, `NavBar`, etc., with session-aware navigation (`tests/ui_tests.rs`).
- **Config Tests**: Verifies environment variable loading and logging setup (`tests/config_tests.rs`).
- **Utils Tests**: Tests error handling, telemetry, caching, and validation (`tests/utils_tests.rs`).

## Benchmarks 📈
Run performance benchmarks:
```shell
cargo bench
```

## License 📄
This project is licensed under the [Open BSV License](LICENSE). See the [LICENSE](LICENSE) file for details.

## Project Structure 🗂️
- **src/auth/**: Authentication logic (`AuthManager`, `OAuthManager`, `PasskeyManager`, `Session`).
- **src/blockchain/**: Wallet and PayMail operations (`WalletManager`, `PaymailManager`, `TransactionManager`).
- **src/config/**: Environment configuration (`EnvConfig`).
- **src/ui/**: Dioxus components (`App`, `Home`, `Auth`, `Profile`, `Settings`, `NavBar`, `ThemeProvider`, `ThemeSwitcher`).
- **src/utils/**: Utilities for logging, telemetry, caching, security, rate limiting, and session management.
- **tests/**: Comprehensive test suites for all modules.

## Contributing 🤝
Contributions are welcome! Please submit pull requests or open issues on [GitHub](https://github.com/murphsicles/zip).

## Contact 📬
For questions, reach out via [GitHub Issues](https://github.com/murphsicles/zip/issues).
