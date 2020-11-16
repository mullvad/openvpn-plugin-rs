# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

### Categories each change fall into

* **Added**: for new features.
* **Changed**: for changes in existing functionality.
* **Deprecated**: for soon-to-be removed features.
* **Removed**: for now removed features.
* **Fixed**: for any bug fixes.
* **Security**: in case of vulnerabilities.


## [Unreleased]


## [0.4.0] - 2020-11-16
### Added
- Add `EventType::AuthFailed` behind a feature (`auth-failed-event`). Allows the plugin to process
  the [Mullvad VPN specific] event in the fork.
- Add event types `ClientConnectDefer` and `ClientConnectDeferV2`, new in OpenVPN 2.5.

[Mullvad VPN specific]: https://github.com/mullvad/openvpn

### Changed
- Upgrade to Rust 2018. New required minimum Rust version (MSRV) is 1.42.0 due to
  #[non_exhaustive] and `proc_macro::TokenStream`.
- Rename `OpenVpnPluginEvent` to `EventType`.
- Rename `EventType::from_int` to `from_repr` and make it return `None` on failure instead of error.
- Make `types` module private and re-export `EventType` plus `EventResult` at crate root.
- Modernize the error types. Remove `Error::description` implementation (in favor of just using
  the `Display` implementation) and change `Error::cause` into `Error::source`.
- Make `EventType` a `#[non_exhaustive]` enum to allow conditionally adding variants with features.
- `EventType` now implements `std::convert::TryFrom` instead of having custom `try_from` method.
  Returns a `Result<Self, i32>` instead of `Option<Self>`.

### Removed
- The `EventType::N` variant. It is not a real event.

### Fixed
- Force the handle type to be the same across all three callback functions in the `openvpn_plugin!`
  macro.


## [0.3.0] - 2017-10-13
### Fixed
- Catch panics from `$open_fn`, `$close_fn` and `$event_fn` instead of unwinding back into C.
- Correctly handle errors in argument and environment parsing, to not panic unwind into C.

### Changed
- Give argument and environment variables by ownership to callbacks instead of just borrowing.
- Make documentation code compile as part of tests, to verify it works.


## [0.2.0] - 2017-07-20
### Added
- `EventResult::Failure` to enable returning `OPENVPN_PLUGIN_FUNC_ERROR` to OpenVPN without logging
  it as an error.

### Changed
- Renamed `SuccessType` to `EventResult`.


## [0.1.0] - 2017-07-19
### Added
- Initial code for easily creating OpenVPN plugins in Rust.
