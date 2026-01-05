# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2](https://github.com/Polymarket/rs-clob-client/compare/v0.3.1...v0.3.2) - 2026-01-04

### Added

- add unknown field warnings for API responses ([#47](https://github.com/Polymarket/rs-clob-client/pull/47))
- *(ws)* add custom feature message types and subscription support ([#79](https://github.com/Polymarket/rs-clob-client/pull/79))

### Fixed

- *(ws)* defer WebSocket connection until first subscription ([#90](https://github.com/Polymarket/rs-clob-client/pull/90))
- *(types)* improve type handling and API compatibility ([#92](https://github.com/Polymarket/rs-clob-client/pull/92))
- add serde aliases for API response field variants ([#88](https://github.com/Polymarket/rs-clob-client/pull/88))
- *(data)* add missing fields to Position and Holder types ([#85](https://github.com/Polymarket/rs-clob-client/pull/85))
- *(gamma)* add missing fields to response types ([#87](https://github.com/Polymarket/rs-clob-client/pull/87))
- *(deser_warn)* show full JSON values in unknown field warnings ([#86](https://github.com/Polymarket/rs-clob-client/pull/86))
- handle order_type field in OpenOrderResponse ([#81](https://github.com/Polymarket/rs-clob-client/pull/81))

### Other

- update README with new features and examples ([#80](https://github.com/Polymarket/rs-clob-client/pull/80))

## [0.3.1](https://github.com/Polymarket/rs-clob-client/compare/v0.3.0...v0.3.1) - 2025-12-31

### Added

- *(ws)* add unsubscribe support with reference counting ([#70](https://github.com/Polymarket/rs-clob-client/pull/70))
- *(auth)* add secret and passphrase accessors to Credentials ([#78](https://github.com/Polymarket/rs-clob-client/pull/78))
- add RTDS (Real-Time Data Socket) client ([#56](https://github.com/Polymarket/rs-clob-client/pull/56))

### Fixed

- *(clob)* align API implementation with OpenAPI spec ([#72](https://github.com/Polymarket/rs-clob-client/pull/72))

### Other

- *(auth)* migrate from sec to secrecy crate ([#75](https://github.com/Polymarket/rs-clob-client/pull/75))
- use re-exported types ([#74](https://github.com/Polymarket/rs-clob-client/pull/74))

## [0.3.0](https://github.com/Polymarket/rs-clob-client/compare/v0.2.1...v0.3.0) - 2025-12-31

### Added

- *(auth)* add key() getter to Credentials ([#69](https://github.com/Polymarket/rs-clob-client/pull/69))
- add geographic restrictions check ([#63](https://github.com/Polymarket/rs-clob-client/pull/63))
- add bridge API client ([#55](https://github.com/Polymarket/rs-clob-client/pull/55))

### Fixed

- *(gamma)* use repeated query params for clob_token_ids ([#65](https://github.com/Polymarket/rs-clob-client/pull/65))
- correct data example required-features name ([#68](https://github.com/Polymarket/rs-clob-client/pull/68))
- *(clob)* allow market orders to supply price ([#67](https://github.com/Polymarket/rs-clob-client/pull/67))
- add CTF Exchange approval to approvals example ([#45](https://github.com/Polymarket/rs-clob-client/pull/45))

### Other

- [**breaking**] ws types ([#52](https://github.com/Polymarket/rs-clob-client/pull/52))
- consolidate request and query params ([#64](https://github.com/Polymarket/rs-clob-client/pull/64))
- [**breaking**] rescope data types and rename feature ([#62](https://github.com/Polymarket/rs-clob-client/pull/62))
- [**breaking**] rescope gamma types ([#61](https://github.com/Polymarket/rs-clob-client/pull/61))
- [**breaking**] scope clob types into request/response ([#60](https://github.com/Polymarket/rs-clob-client/pull/60))
- [**breaking**] WS cleanup ([#58](https://github.com/Polymarket/rs-clob-client/pull/58))
- [**breaking**] minor cleanup ([#57](https://github.com/Polymarket/rs-clob-client/pull/57))

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
