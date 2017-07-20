# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2017-07-20
### Added
- `EventResult::Failure` to enable returning `OPENVPN_PLUGIN_FUNC_ERROR` to OpenVPN without logging
  it as an error.

### Changed
- Renamed `SuccessType` to `EventResult`.

## [0.1.0] - 2017-07-19
### Added
- Initial code for easily creating OpenVPN plugins in Rust.
