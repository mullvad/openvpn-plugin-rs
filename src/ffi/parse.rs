// Copyright 2017 Amagicom AB.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashMap;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw::c_char;
use std::str::Utf8Error;

/// Error type returned by the ffi parsing functions if the input data is invalid in some way.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ParseError {
    /// Given pointer is a null pointer.
    NullPtr,
    /// A string in the environment has no '=' char in it, and is thus not a valid environment
    /// entry.
    NoEqual(CString),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ParseError::NullPtr => f.write_str(self.description()),
            ParseError::NoEqual(ref s) => write!(f, "No equal sign in \"{}\"", s.to_string_lossy()),
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::NullPtr => "Input is null pointer",
            ParseError::NoEqual(_) => "No equal sign in string",
        }
    }
}


/// Parses a null-terminated C string array into a Vec<CString> for safe usage.
///
/// Returns an Err if given a null pointer.
///
/// # Segfaults
///
/// Can cause the program to crash if the pointer array starting at `ptr` is not correctly null
/// terminated. Likewise, if any string pointed to is not properly null-terminated it may crash.
pub unsafe fn string_array(mut ptr: *const *const c_char) -> Result<Vec<CString>, ParseError> {
    if ptr.is_null() {
        Err(ParseError::NullPtr)
    } else {
        let mut strings = Vec::new();
        while !(*ptr).is_null() {
            strings.push(CStr::from_ptr(*ptr).to_owned());
            ptr = ptr.offset(1);
        }
        Ok(strings)
    }
}

/// Convenience method for plugins to convert the C string arrays they are given into real Rust
/// strings.
pub fn string_array_utf8(strings: &[CString]) -> Result<Vec<String>, Utf8Error> {
    strings
        .iter()
        .map(|s| s.to_str().map(|s| s.to_owned()))
        .collect()
}


/// Parses a null-terminated array of C strings with "=" delimiters into a key-value map.
///
/// The input environment has to contain null-terminated strings containing at least
/// one equal sign ("="). Every string is split at the first equal sign and added to the map with
/// the first part being the key and the second the value.
///
/// If multiple entries have the same key, the last one will be in the result map.
///
/// # Segfaults
///
/// Uses `string_array` internally and will segfault for the same reasons as that function.
pub unsafe fn env(envptr: *const *const c_char) -> Result<HashMap<CString, CString>, ParseError> {
    let mut map = HashMap::new();
    for string in string_array(envptr)? {
        let string_bytes = string.as_bytes();
        let equal_index = string_bytes
            .iter()
            .position(|&c| c == b'=')
            .ok_or_else(|| ParseError::NoEqual(string.clone()))?;

        // It's safe to unwrap since CString guarantees no null bytes.
        let key = CString::new(&string_bytes[..equal_index]).unwrap();
        let value = CString::new(&string_bytes[equal_index + 1..]).unwrap();
        map.insert(key, value);
    }
    Ok(map)
}

/// Convenience method for plugins to convert the environments given to them into Rust String based
/// environments.
pub fn env_utf8(env: &HashMap<CString, CString>) -> Result<HashMap<String, String>, Utf8Error> {
    let mut output_env = HashMap::with_capacity(env.len());
    for (key, value) in env {
        output_env.insert(key.to_str()?.to_owned(), value.to_str()?.to_owned());
    }
    Ok(output_env)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_char;
    use std::ptr;

    #[test]
    fn string_array_null() {
        assert_eq!(Err(ParseError::NullPtr), unsafe {
            string_array(ptr::null())
        });
    }

    #[test]
    fn string_array_empty() {
        let ptr_arr = [ptr::null()];
        let result = unsafe { string_array(&ptr_arr as *const *const c_char).unwrap() };
        assert!(result.is_empty());
    }

    #[test]
    fn string_array_no_space_trim() {
        let test_str = " foobar \0";
        let ptr_arr = [test_str as *const _ as *const c_char, ptr::null()];
        let result = unsafe { string_array(&ptr_arr as *const *const c_char).unwrap() };
        assert_eq!([CString::new(" foobar ").unwrap()], &result[..]);
    }

    #[test]
    fn string_array_two_strings() {
        let test_str1 = "foo\0";
        let test_str2 = "bar\0";
        let ptr_arr = [
            test_str1 as *const _ as *const c_char,
            test_str2 as *const _ as *const c_char,
            ptr::null(),
        ];
        let result = unsafe { string_array(&ptr_arr as *const *const c_char).unwrap() };
        assert_eq!(
            [CString::new("foo").unwrap(), CString::new("bar").unwrap()],
            &result[..]
        );
    }

    #[test]
    fn string_array_utf8_happy_path() {
        let array = &[CString::new("foo").unwrap(), CString::new("bar").unwrap()];
        let result = string_array_utf8(array).unwrap();
        assert_eq!(vec!["foo", "bar"], result);
    }

    #[test]
    fn string_array_utf8_invalid() {
        // 192 is not a valid utf8 byte
        let array = &[CString::new(vec![192]).unwrap()];
        assert!(string_array_utf8(array).is_err());
    }

    #[test]
    fn env_one_value() {
        let test_str = "var_a=value_b\0";
        let key = CString::new("var_a").unwrap();
        let value = CString::new("value_b").unwrap();

        let ptr_arr = [test_str as *const _ as *const c_char, ptr::null()];
        let result = unsafe { env(&ptr_arr as *const *const c_char).unwrap() };
        assert_eq!(1, result.len());
        assert_eq!(Some(&value), result.get(&key));
    }

    #[test]
    fn env_no_equal() {
        let test_str = "foobar\0";
        let ptr_arr = [test_str as *const _ as *const c_char, ptr::null()];
        let result = unsafe { env(&ptr_arr as *const *const c_char) };
        assert_eq!(
            result,
            Err(ParseError::NoEqual(CString::new("foobar").unwrap()))
        );
    }

    #[test]
    fn env_double_equal() {
        let test_str = "foo=bar=baz\0";
        let key = CString::new("foo").unwrap();
        let value = CString::new("bar=baz").unwrap();

        let ptr_arr = [test_str as *const _ as *const c_char, ptr::null()];
        let env = unsafe { env(&ptr_arr as *const *const c_char).unwrap() };
        assert_eq!(1, env.len());
        assert_eq!(Some(&value), env.get(&key));
    }

    #[test]
    fn env_two_same_key() {
        let test_str1 = "foo=123\0";
        let test_str2 = "foo=abc\0";
        let ptr_arr = [
            test_str1 as *const _ as *const c_char,
            test_str2 as *const _ as *const c_char,
            ptr::null(),
        ];
        let key = CString::new("foo").unwrap();
        let value = CString::new("abc").unwrap();

        let env = unsafe { env(&ptr_arr as *const *const c_char).unwrap() };
        assert_eq!(1, env.len());
        assert_eq!(Some(&value), env.get(&key));
    }

    #[test]
    fn env_utf8_happy_path() {
        let mut env = HashMap::new();
        env.insert(CString::new("foo").unwrap(), CString::new("bar").unwrap());
        env.insert(CString::new("baz").unwrap(), CString::new("123").unwrap());
        let result = env_utf8(&env).unwrap();
        assert_eq!("bar", result.get("foo").unwrap());
        assert_eq!("123", result.get("baz").unwrap());
    }

    #[test]
    fn env_utf8_invalid() {
        // 192 is not a valid utf8 byte
        let mut env = HashMap::new();
        env.insert(
            CString::new(vec![192]).unwrap(),
            CString::new("bar").unwrap(),
        );
        assert!(env_utf8(&env).is_err());
    }
}
