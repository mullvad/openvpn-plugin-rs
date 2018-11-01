// Copyright 2017 Amagicom AB.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Constants for OpenVPN. Taken from include/openvpn-plugin.h in the OpenVPN repository:
//! https://github.com/OpenVPN/openvpn/blob/master/include/openvpn-plugin.h.in

use std::error;
use std::fmt;
use std::os::raw::c_int;

/// Error thrown when trying to convert from an invalid integer into an `OpenVpnPluginEvent`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InvalidEnumVariant(c_int);

impl fmt::Display for InvalidEnumVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} is not a valid OPENVPN_PLUGIN_* constant", self.0)
    }
}

impl error::Error for InvalidEnumVariant {
    fn description(&self) -> &str {
        "Integer does not match any enum variant"
    }
}


/// Enum whose variants correspond to the `OPENVPN_PLUGIN_*` event constants.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OpenVpnPluginEvent {
    Up = 0,
    Down = 1,
    RouteUp = 2,
    IpChange = 3,
    TlsVerify = 4,
    AuthUserPassVerify = 5,
    ClientConnect = 6,
    ClientDisconnect = 7,
    LearnAddress = 8,
    ClientConnectV2 = 9,
    TlsFinal = 10,
    EnablePf = 11,
    RoutePredown = 12,
    N = 13,
}

impl OpenVpnPluginEvent {
    /// Tries to parse an integer from C into a variant of `OpenVpnPluginEvent`.
    pub fn from_int(i: c_int) -> Result<OpenVpnPluginEvent, InvalidEnumVariant> {
        if i >= OpenVpnPluginEvent::Up as c_int && i <= OpenVpnPluginEvent::N as c_int {
            Ok(unsafe { ::std::mem::transmute_copy::<c_int, OpenVpnPluginEvent>(&i) })
        } else {
            Err(InvalidEnumVariant(i))
        }
    }
}

/// Translates a collection of `OpenVpnPluginEvent` instances into a bitmask in the format OpenVPN
/// expects it in `type_mask`.
pub fn events_to_bitmask(events: &[OpenVpnPluginEvent]) -> c_int {
    let mut bitmask: c_int = 0;
    for event in events {
        bitmask |= 1 << (*event as i32);
    }
    bitmask
}


/// Enum representing the results an OpenVPN plugin can return from an event callback.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum EventResult {
    /// Will return `OPENVPN_PLUGIN_FUNC_SUCCESS` to OpenVPN.
    /// Indicates that the plugin marks the event as a success. This means an auth is approved
    /// or similar, depending on which type of event.
    Success,

    /// Will return `OPENVPN_PLUGIN_FUNC_DEFERRED` to OpenVPN.
    /// WARNING: Can only be returned from the `OpenVpnPluginEvent::AuthUserPassVerify`
    /// (`OPENVPN_PLUGIN_AUTH_USER_PASS_VERIFY`) event. No other events may return this variant.
    /// Returning this tells OpenVPN to continue its normal work and that the decision on if the
    /// authentication is accepted or not will be delivered later, via writing to the path under
    /// the `auth_control_file` environment variable.
    Deferred,

    /// Will return `OPENVPN_PLUGIN_FUNC_ERROR` to OpenVPN.
    /// Both returning `Ok(EventResult::Failure)` and `Err(e)` from a callback will result in
    /// `OPENVPN_PLUGIN_FUNC_ERROR` being returned to OpenVPN. The difference being that an
    /// `Err(e)` will also log the error `e`. This variant is intended for when the plugin did
    /// not encounter an error, but the event is a failure or is to be declined. Intended to be
    /// used to decline an authentication request and similar.
    Failure,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_int_first() {
        assert_eq!(OpenVpnPluginEvent::from_int(0), Ok(OpenVpnPluginEvent::Up));
    }

    #[test]
    fn from_int_last() {
        assert_eq!(OpenVpnPluginEvent::from_int(13), Ok(OpenVpnPluginEvent::N));
    }

    #[test]
    fn from_int_all_valid() {
        for i in 0..13 {
            if OpenVpnPluginEvent::from_int(i).is_err() {
                panic!("{} not covered", i);
            }
        }
    }

    #[test]
    fn from_int_negative() {
        let result = OpenVpnPluginEvent::from_int(-5);
        assert_eq!(result, Err(InvalidEnumVariant(-5)));
    }

    #[test]
    fn from_int_invalid() {
        let result = OpenVpnPluginEvent::from_int(14);
        assert_eq!(result, Err(InvalidEnumVariant(14)));
    }

    #[test]
    fn event_enum_to_str() {
        let result = format!("{:?}", OpenVpnPluginEvent::Up);
        assert_eq!("Up", result);
    }

    #[test]
    fn events_to_bitmask_no_events() {
        let result = events_to_bitmask(&[]);
        assert_eq!(0, result);
    }

    #[test]
    fn events_to_bitmask_one_event() {
        let result = events_to_bitmask(&[OpenVpnPluginEvent::Up]);
        assert_eq!(0b1, result);
    }

    #[test]
    fn events_to_bitmask_another_event() {
        let result = events_to_bitmask(&[OpenVpnPluginEvent::RouteUp]);
        assert_eq!(0b100, result);
    }

    #[test]
    fn events_to_bitmask_many_events() {
        let result = events_to_bitmask(&[OpenVpnPluginEvent::RouteUp, OpenVpnPluginEvent::N]);
        assert_eq!((1 << 13) | (1 << 2), result);
    }
}
