/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `ByteString` struct.

use cssparser::CowRcStr;
use html5ever::{LocalName, Namespace};
use servo_atoms::Atom;
use std::borrow::{Borrow, Cow, ToOwned};
use std::default::Default;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops;
use std::ops::{Deref, DerefMut};
use std::str;
use std::str::{Bytes, FromStr};

/// Encapsulates the IDL `ByteString` type.
#[derive(Clone, Debug, Default, Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub struct ByteString(Vec<u8>);

impl ByteString {
    /// Creates a new `ByteString`.
    pub fn new(value: Vec<u8>) -> ByteString {
        ByteString(value)
    }

    /// Returns `self` as a string, if it encodes valid UTF-8, and `None`
    /// otherwise.
    pub fn as_str(&self) -> Option<&str> {
        str::from_utf8(&self.0).ok()
    }

    /// Returns the length.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Compare `self` to `other`, matching A–Z and a–z as equal.
    pub fn eq_ignore_case(&self, other: &ByteString) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }

    /// Returns `self` with A–Z replaced by a–z.
    pub fn to_lower(&self) -> ByteString {
        ByteString::new(self.0.to_ascii_lowercase())
    }
}

impl Into<Vec<u8>> for ByteString {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl Hash for ByteString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl FromStr for ByteString {
    type Err = ();
    fn from_str(s: &str) -> Result<ByteString, ()> {
        Ok(ByteString::new(s.to_owned().into_bytes()))
    }
}

impl ops::Deref for ByteString {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

/// A string that is constructed from a UCS-2 buffer by replacing invalid code
/// points with the replacement character.
#[derive(Clone, Default, MallocSizeOf)]
pub struct USVString(pub String);


/// Returns whether `s` is a `token`, as defined by
/// [RFC 2616](http://tools.ietf.org/html/rfc2616#page-17).
pub fn is_token(s: &[u8]) -> bool {
    if s.is_empty() {
        return false; // A token must be at least a single character
    }
    s.iter().all(|&x| {
        // http://tools.ietf.org/html/rfc2616#section-2.2
        match x {
            0...31 | 127 => false, // CTLs
            40 |
            41 |
            60 |
            62 |
            64 |
            44 |
            59 |
            58 |
            92 |
            34 |
            47 |
            91 |
            93 |
            63 |
            61 |
            123 |
            125 |
            32 => false, // separators
            x if x > 127 => false, // non-CHARs
            _ => true,
        }
    })
}


/// A DOMString.
///
/// This type corresponds to the [`DOMString`](idl) type in WebIDL.
///
/// [idl]: https://heycam.github.io/webidl/#idl-DOMString
///
/// Conceptually, a DOMString has the same value space as a JavaScript String,
/// i.e., an array of 16-bit *code units* representing UTF-16, potentially with
/// unpaired surrogates present (also sometimes called WTF-16).
///
/// Currently, this type stores a Rust `String`, in order to avoid issues when
/// integrating with the rest of the Rust ecosystem and even the rest of the
/// browser itself.
///
/// However, Rust `String`s are guaranteed to be valid UTF-8, and as such have
/// a *smaller value space* than WTF-16 (i.e., some JavaScript String values
/// can not be represented as a Rust `String`). This introduces the question of
/// what to do with values being passed from JavaScript to Rust that contain
/// unpaired surrogates.
///
/// The hypothesis is that it does not matter much how exactly those values are
/// transformed, because passing unpaired surrogates into the DOM is very rare.
/// In order to test this hypothesis, Servo will panic when encountering any
/// unpaired surrogates on conversion to `DOMString` by default. (The command
/// line option `-Z replace-surrogates` instead causes Servo to replace the
/// unpaired surrogate by a U+FFFD replacement character.)
///
/// Currently, the lack of crash reports about this issue provides some
/// evidence to support the hypothesis. This evidence will hopefully be used to
/// convince other browser vendors that it would be safe to replace unpaired
/// surrogates at the boundary between JavaScript and native code. (This would
/// unify the `DOMString` and `USVString` types, both in the WebIDL standard
/// and in Servo.)
///
/// This type is currently `!Send`, in order to help with an independent
/// experiment to store `JSString`s rather than Rust `String`s.
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub struct DOMString(String, PhantomData<*const ()>);

impl DOMString {
    /// Creates a new `DOMString`.
    pub fn new() -> DOMString {
        DOMString(String::new(), PhantomData)
    }

    /// Creates a new `DOMString` from a `String`.
    pub fn from_string(s: String) -> DOMString {
        DOMString(s, PhantomData)
    }

    /// Appends a given string slice onto the end of this String.
    pub fn push_str(&mut self, string: &str) {
        self.0.push_str(string)
    }

    /// Clears this `DOMString`, removing all contents.
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Shortens this String to the specified length.
    pub fn truncate(&mut self, new_len: usize) {
        self.0.truncate(new_len);
    }

    /// An iterator over the bytes of this `DOMString`.
    pub fn bytes(&self) -> Bytes {
        self.0.bytes()
    }

    /// Removes newline characters according to <https://infra.spec.whatwg.org/#strip-newlines>.
    pub fn strip_newlines(&mut self) {
        self.0.retain(|c| c != '\r' && c != '\n');
    }

    /// Removes leading and trailing ASCII whitespaces according to
    /// <https://infra.spec.whatwg.org/#strip-leading-and-trailing-ascii-whitespace>.
    pub fn strip_leading_and_trailing_ascii_whitespace(&mut self) {
        if self.0.len() == 0 { return; }

        let last_non_whitespace = match self.0.rfind(|ref c| !char::is_ascii_whitespace(c)) {
            Some(idx) => idx + 1,
            None => {
                self.0.clear();
                return;
            }
        };
        let first_non_whitespace = self.0.find(|ref c| !char::is_ascii_whitespace(c)).unwrap();

        self.0.truncate(last_non_whitespace);
        let _ = self.0.splice(0..first_non_whitespace, "");
    }

    /// Validates this `DOMString` is a time string according to
    /// <https://html.spec.whatwg.org/multipage/#valid-time-string>.
    pub fn is_valid_time_string(&self) -> bool {
        enum State {
            HourHigh,
            HourLow09,
            HourLow03,
            MinuteColon,
            MinuteHigh,
            MinuteLow,
            SecondColon,
            SecondHigh,
            SecondLow,
            MilliStop,
            MilliHigh,
            MilliMiddle,
            MilliLow,
            Done,
            Error,
        }
        let next_state = |valid: bool, next: State| -> State { if valid { next } else { State::Error } };

        let state = self.chars().fold(State::HourHigh, |state, c| {
            match state {
                // Step 1 "HH"
                State::HourHigh => {
                    match c {
                        '0' | '1' => State::HourLow09,
                        '2' => State::HourLow03,
                        _ => State::Error,
                    }
                },
                State::HourLow09 => next_state(c.is_digit(10), State::MinuteColon),
                State::HourLow03 => next_state(c.is_digit(4), State::MinuteColon),

                // Step 2 ":"
                State::MinuteColon => next_state(c == ':', State::MinuteHigh),

                // Step 3 "mm"
                State::MinuteHigh => next_state(c.is_digit(6), State::MinuteLow),
                State::MinuteLow => next_state(c.is_digit(10), State::SecondColon),

                // Step 4.1 ":"
                State::SecondColon => next_state(c == ':', State::SecondHigh),
                // Step 4.2 "ss"
                State::SecondHigh => next_state(c.is_digit(6), State::SecondLow),
                State::SecondLow => next_state(c.is_digit(10), State::MilliStop),

                // Step 4.3.1 "."
                State::MilliStop => next_state(c == '.', State::MilliHigh),
                // Step 4.3.2 "SSS"
                State::MilliHigh => next_state(c.is_digit(6), State::MilliMiddle),
                State::MilliMiddle => next_state(c.is_digit(10), State::MilliLow),
                State::MilliLow => next_state(c.is_digit(10), State::Done),

                _ => State::Error,
            }
        });

        match state {
            State::Done |
            // Step 4 (optional)
            State::SecondColon |
            // Step 4.3 (optional)
            State::MilliStop |
            // Step 4.3.2 (only 1 digit required)
            State::MilliMiddle | State::MilliLow => true,
            _ => false
        }
    }
}

impl Borrow<str> for DOMString {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Default for DOMString {
    fn default() -> Self {
        DOMString(String::new(), PhantomData)
    }
}

impl Deref for DOMString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        &self.0
    }
}

impl DerefMut for DOMString {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl AsRef<str> for DOMString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DOMString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl PartialEq<str> for DOMString {
    fn eq(&self, other: &str) -> bool {
        &**self == other
    }
}

impl<'a> PartialEq<&'a str> for DOMString {
    fn eq(&self, other: &&'a str) -> bool {
        &**self == *other
    }
}

impl From<String> for DOMString {
    fn from(contents: String) -> DOMString {
        DOMString(contents, PhantomData)
    }
}

impl<'a> From<&'a str> for DOMString {
    fn from(contents: &str) -> DOMString {
        DOMString::from(String::from(contents))
    }
}

impl<'a> From<Cow<'a, str>> for DOMString {
    fn from(contents: Cow<'a, str>) -> DOMString {
        match contents {
            Cow::Owned(s) => DOMString::from(s),
            Cow::Borrowed(s) => DOMString::from(s),
        }
    }
}

impl From<DOMString> for LocalName {
    fn from(contents: DOMString) -> LocalName {
        LocalName::from(contents.0)
    }
}

impl From<DOMString> for Namespace {
    fn from(contents: DOMString) -> Namespace {
        Namespace::from(contents.0)
    }
}

impl From<DOMString> for Atom {
    fn from(contents: DOMString) -> Atom {
        Atom::from(contents.0)
    }
}

impl From<DOMString> for String {
    fn from(contents: DOMString) -> String {
        contents.0
    }
}

impl Into<Vec<u8>> for DOMString {
    fn into(self) -> Vec<u8> {
        self.0.into()
    }
}

impl<'a> Into<Cow<'a, str>> for DOMString {
    fn into(self) -> Cow<'a, str> {
        self.0.into()
    }
}

impl<'a> Into<CowRcStr<'a>> for DOMString {
    fn into(self) -> CowRcStr<'a> {
        self.0.into()
    }
}

impl Extend<char> for DOMString {
    fn extend<I>(&mut self, iterable: I) where I: IntoIterator<Item=char> {
        self.0.extend(iterable)
    }
}
