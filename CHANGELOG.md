# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
