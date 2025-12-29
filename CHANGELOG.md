# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1](https://github.com/Polymarket/rs-clob-client/compare/v0.2.0...v0.2.1) - 2025-12-29

### Added

- complete gamma client ([#40](https://github.com/Polymarket/rs-clob-client/pull/40))
- add data-api client ([#39](https://github.com/Polymarket/rs-clob-client/pull/39))

### Fixed

- use TryFrom for TickSize to avoid panic on unknown values ([#43](https://github.com/Polymarket/rs-clob-client/pull/43))

### Other

- *(cargo)* bump tracing from 0.1.41 to 0.1.44 ([#49](https://github.com/Polymarket/rs-clob-client/pull/49))
- *(cargo)* bump serde_json from 1.0.146 to 1.0.148 ([#51](https://github.com/Polymarket/rs-clob-client/pull/51))
- *(cargo)* bump alloy from 1.1.3 to 1.2.1 ([#50](https://github.com/Polymarket/rs-clob-client/pull/50))
- *(cargo)* bump reqwest from 0.12.27 to 0.12.28 ([#48](https://github.com/Polymarket/rs-clob-client/pull/48))

## [0.2.0](https://github.com/Polymarket/rs-clob-client/compare/v0.1.2...v0.2.0) - 2025-12-27

### Added

- WebSocket client for real-time market and user data ([#26](https://github.com/Polymarket/rs-clob-client/pull/26))

### Other

- [**breaking**] change from `derive_builder` to `bon` ([#41](https://github.com/Polymarket/rs-clob-client/pull/41))

## [0.1.2](https://github.com/Polymarket/rs-clob-client/compare/v0.1.1...v0.1.2) - 2025-12-23

### Added

- add optional tracing instrumentation ([#38](https://github.com/Polymarket/rs-clob-client/pull/38))
- add gamma client ([#31](https://github.com/Polymarket/rs-clob-client/pull/31))
- support share-denominated market orders ([#29](https://github.com/Polymarket/rs-clob-client/pull/29))

### Fixed

- mask salt for limit orders ([#30](https://github.com/Polymarket/rs-clob-client/pull/30))
- mask salt to 53 bits ([#27](https://github.com/Polymarket/rs-clob-client/pull/27))

### Other

- rescope clients with gamma feature ([#37](https://github.com/Polymarket/rs-clob-client/pull/37))
- Replacing `status: String` to enum ([#36](https://github.com/Polymarket/rs-clob-client/pull/36))
- *(cargo)* bump serde_json from 1.0.145 to 1.0.146 ([#34](https://github.com/Polymarket/rs-clob-client/pull/34))
- *(cargo)* bump reqwest from 0.12.26 to 0.12.27 ([#33](https://github.com/Polymarket/rs-clob-client/pull/33))
- *(gha)* bump dtolnay/rust-toolchain from 0b1efabc08b657293548b77fb76cc02d26091c7e to f7ccc83f9ed1e5b9c81d8a67d7ad1a747e22a561 ([#32](https://github.com/Polymarket/rs-clob-client/pull/32))

## [0.1.1](https://github.com/Polymarket/rs-clob-client/compare/v0.1.0...v0.1.1) - 2025-12-17

### Fixed

- remove signer from Authenticated ([#22](https://github.com/Polymarket/rs-clob-client/pull/22))

### Other

- enable release-plz ([#23](https://github.com/Polymarket/rs-clob-client/pull/23))
- add crates.io badge ([#20](https://github.com/Polymarket/rs-clob-client/pull/20))
