// Copyright 2023 Mullvad VPN AB.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This debug/example OpenVPN plugin listens for almost all events and prints the arguments
//! for each event callback and returns success in every case.

use openvpn_plugin::{EventResult, EventType};
use std::collections::HashMap;
use std::ffi::CString;

/// The list of OpenVPN events we register for. The list contains all possible events, but some of
/// them are commented out since they work slightly different, and will not work with the simple
/// log-and-return-success implementation we have here.
pub static INTERESTING_EVENTS: &[EventType] = &[
    EventType::Up,
    EventType::Down,
    EventType::RouteUp,
    EventType::IpChange,
    // EventType::TlsVerify,
    // EventType::AuthUserPassVerify,
    EventType::ClientConnect,
    EventType::ClientDisconnect,
    EventType::LearnAddress,
    EventType::ClientConnectV2,
    EventType::TlsFinal,
    EventType::EnablePf,
    EventType::RoutePredown,
    EventType::ClientConnectDefer,
    EventType::ClientConnectDeferV2,
    #[cfg(feature = "auth-failed-event")]
    EventType::AuthFailed,
];

openvpn_plugin::openvpn_plugin!(
    crate::debug_open,
    crate::lol::debug_close,
    crate::debug_event,
    ()
);

fn debug_open(
    args: Vec<CString>,
    env: HashMap<CString, CString>,
) -> Result<(Vec<EventType>, ()), ::std::io::Error> {
    println!(
        "DEBUG-PLUGIN: open called:\n\targs: {:?}\n\tenv: {:?}",
        args, env
    );
    Ok((INTERESTING_EVENTS.to_vec(), ()))
}

mod lol {
    pub fn debug_close(_handle: ()) {
        println!("DEBUG-PLUGIN: close called")
    }
}

fn debug_event(
    event: EventType,
    args: Vec<CString>,
    env: HashMap<CString, CString>,
    _handle: &mut (),
) -> Result<EventResult, ::std::io::Error> {
    println!(
        "DEBUG-PLUGIN: event called:\n\tevent: {:?}\n\targs: {:?}\n\tenv: {:?}",
        event, args, env
    );
    Ok(EventResult::Success)
}
