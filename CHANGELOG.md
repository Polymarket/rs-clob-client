# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/Polymarket/rs-clob-client/compare/v0.3.3...v0.4.0) - 2026-01-12

### Added

- *(clob)* add cache setter methods to prewarm market data ([#153](https://github.com/Polymarket/rs-clob-client/pull/153))
- *(bridge)* improve bridge type safety ([#151](https://github.com/Polymarket/rs-clob-client/pull/151))
- *(gamma)* convert neg_risk_market_id and neg_risk_request_id to B256 ([#143](https://github.com/Polymarket/rs-clob-client/pull/143))
- *(gamma)* convert question_id fields to B256 type ([#142](https://github.com/Polymarket/rs-clob-client/pull/142))
- *(clob)* clob typed b256 address ([#139](https://github.com/Polymarket/rs-clob-client/pull/139))
- *(clob)* add clob feature flag for optional CLOB compilation ([#135](https://github.com/Polymarket/rs-clob-client/pull/135))
- *(tracing)* add serde_path_to_error for detailed deserialization on errors ([#140](https://github.com/Polymarket/rs-clob-client/pull/140))
- *(data)* use typed Address and B256 for hex string fields, update data example ([#132](https://github.com/Polymarket/rs-clob-client/pull/132))
- *(gamma)* use typed Address and B256 for hex string fields ([#126](https://github.com/Polymarket/rs-clob-client/pull/126))
- *(ctf)* add CTF client/operations ([#82](https://github.com/Polymarket/rs-clob-client/pull/82))
- add Unknown(String) variant to all enums for forward compatibility ([#124](https://github.com/Polymarket/rs-clob-client/pull/124))
- add subscribe_last_trade_price websocket method ([#121](https://github.com/Polymarket/rs-clob-client/pull/121))
- support post-only orders ([#115](https://github.com/Polymarket/rs-clob-client/pull/115))
- *(heartbeats)* [**breaking**] add heartbeats ([#113](https://github.com/Polymarket/rs-clob-client/pull/113))

### Fixed

- *(rfq)* url path fixes ([#162](https://github.com/Polymarket/rs-clob-client/pull/162))
- *(gamma)* use repeated query params for array fields ([#148](https://github.com/Polymarket/rs-clob-client/pull/148))
- *(rtds)* serialize Chainlink filters as JSON string ([#136](https://github.com/Polymarket/rs-clob-client/pull/136)) ([#137](https://github.com/Polymarket/rs-clob-client/pull/137))
- add missing makerRebatesFeeShareBps field to Market struct ([#130](https://github.com/Polymarket/rs-clob-client/pull/130))
- add MakerRebate enum option to ActivityType ([#127](https://github.com/Polymarket/rs-clob-client/pull/127))
- suppress unused variable warnings in tracing cfg blocks ([#125](https://github.com/Polymarket/rs-clob-client/pull/125))
- add Yield enum option to ActivityType ([#122](https://github.com/Polymarket/rs-clob-client/pull/122))

### Other

- *(rtds)* [**breaking**] well-type RTDS structs ([#167](https://github.com/Polymarket/rs-clob-client/pull/167))
- *(gamma)* [**breaking**] well-type structs ([#166](https://github.com/Polymarket/rs-clob-client/pull/166))
- *(clob/rfq)* well-type structs ([#163](https://github.com/Polymarket/rs-clob-client/pull/163))
- *(data)* well-type data types ([#159](https://github.com/Polymarket/rs-clob-client/pull/159))
- *(gamma,rtds)* add Builder to non_exhaustive structs ([#160](https://github.com/Polymarket/rs-clob-client/pull/160))
- *(ctf)* add Builder to non_exhaustive response structs ([#161](https://github.com/Polymarket/rs-clob-client/pull/161))
- *(ws)* [**breaking**] well-type ws structs ([#156](https://github.com/Polymarket/rs-clob-client/pull/156))
- add benchmarks for CLOB and WebSocket types/operations ([#155](https://github.com/Polymarket/rs-clob-client/pull/155))
- *(clob)* [**breaking**] well-type requests/responses with U256 ([#150](https://github.com/Polymarket/rs-clob-client/pull/150))
- update rustdocs ([#134](https://github.com/Polymarket/rs-clob-client/pull/134))
- *(ws)* extract WsError to shared ws module ([#131](https://github.com/Polymarket/rs-clob-client/pull/131))
- update license ([#128](https://github.com/Polymarket/rs-clob-client/pull/128))
- update builder method doc comment ([#129](https://github.com/Polymarket/rs-clob-client/pull/129))

## [0.3.3](https://github.com/Polymarket/rs-clob-client/compare/v0.3.2...v0.3.3) - 2026-01-06

### Added

- *(auth)* auto derive funder address ([#99](https://github.com/Polymarket/rs-clob-client/pull/99))
- *(rfq)* add standalone RFQ API client ([#76](https://github.com/Polymarket/rs-clob-client/pull/76))
- *(types)* re-export commonly used external types for API ergonomics ([#102](https://github.com/Polymarket/rs-clob-client/pull/102))

### Fixed

- add missing cumulativeMarkets field to Event struct ([#108](https://github.com/Polymarket/rs-clob-client/pull/108))

### Other

- *(cargo)* bump reqwest from 0.12.28 to 0.13.1 ([#103](https://github.com/Polymarket/rs-clob-client/pull/103))
- *(ws)* common connection for clob ws and rtds ([#97](https://github.com/Polymarket/rs-clob-client/pull/97))
- *(cargo)* bump tokio from 1.48.0 to 1.49.0 ([#104](https://github.com/Polymarket/rs-clob-client/pull/104))
- *(examples)* improve approvals example with tracing ([#101](https://github.com/Polymarket/rs-clob-client/pull/101))
- *(examples)* improve bridge example with tracing ([#100](https://github.com/Polymarket/rs-clob-client/pull/100))
- *(examples)* improve rtds example with tracing and dynamic IDs ([#94](https://github.com/Polymarket/rs-clob-client/pull/94))
- *(examples)* improve gamma example with tracing and dynamic IDs ([#93](https://github.com/Polymarket/rs-clob-client/pull/93))

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
