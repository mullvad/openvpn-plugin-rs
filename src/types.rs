// Copyright 2017 Amagicom AB.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Constants for OpenVPN. Taken from include/openvpn-plugin.h in the OpenVPN repository:
//! https://github.com/OpenVPN/openvpn/blob/master/include/openvpn-plugin.h.in

use std::os::raw::c_int;

use derive_try_from_primitive::TryFromPrimitive;


/// All the events that an OpenVPN plugin can register for and get notified about.
/// This is a Rust representation of the constants named `OPENVPN_PLUGIN_*` in `openvpn-plugin.h`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, TryFromPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
#[repr(i32)]
pub enum EventType {
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
    //EnablePf = 11, // feature has been removed as of OpenVPN 2.6
    RoutePredown = 12,
    ClientConnectDefer = 13,
    ClientConnectDeferV2 = 14,
    ClientCrresponse = 15,
    #[cfg(feature = "auth-failed-event")]
    AuthFailed = 16,
}

/// Translates a collection of `EventType` instances into a bitmask in the format OpenVPN
/// expects it in `type_mask`.
pub fn events_to_bitmask(events: &[EventType]) -> c_int {
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
    /// WARNING: Can only be returned from the `EventType::AuthUserPassVerify`
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
    use std::convert::TryFrom;

    #[test]
    fn event_enum_to_str() {
        let result = format!("{:?}", EventType::Up);
        assert_eq!("Up", result);
    }

    #[test]
    fn events_to_bitmask_no_events() {
        let result = events_to_bitmask(&[]);
        assert_eq!(0, result);
    }

    #[test]
    fn events_to_bitmask_one_event() {
        let result = events_to_bitmask(&[EventType::Up]);
        assert_eq!(0b1, result);
    }

    #[test]
    fn events_to_bitmask_another_event() {
        let result = events_to_bitmask(&[EventType::RouteUp]);
        assert_eq!(0b100, result);
    }

    #[test]
    fn events_to_bitmask_many_events() {
        let result = events_to_bitmask(&[EventType::RouteUp, EventType::RoutePredown]);
        assert_eq!((1 << 12) | (1 << 2), result);
    }

    #[test]
    fn events_max_value() {
        let auth_failed = EventType::try_from(15);
        #[cfg(feature = "auth-failed-event")]
        assert_eq!(auth_failed.unwrap(), EventType::AuthFailed);
        #[cfg(not(feature = "auth-failed-event"))]
        assert_eq!(auth_failed, Err(15));

        assert_eq!(EventType::try_from(16), Err(16));
    }
}
