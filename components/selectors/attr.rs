/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parser::SelectorImpl;
use std::ascii::AsciiExt;

pub enum AttrSelectorOperation<'a, Impl: SelectorImpl> where Impl::AttrValue: 'a {
    Exists,
    WithValue {
        operator: AttrSelectorOperator,
        case_sensitivity: CaseSensitivity,
        expected_value: &'a Impl::AttrValue,
    }
}

impl<'a, Impl: SelectorImpl> AttrSelectorOperation<'a, Impl> {
    pub fn eval_str(&self, element_attr_value: &str) -> bool
    where Impl::AttrValue: AsRef<str> {
        match *self {
            AttrSelectorOperation::Exists => true,
            AttrSelectorOperation::WithValue { operator, case_sensitivity, expected_value } => {
                operator.eval_str(element_attr_value, expected_value.as_ref(), case_sensitivity)
            }
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum AttrSelectorOperator {
    Equal,  // =
    Includes,  // ~=
    DashMatch,  // |=
    Prefix,  // ^=
    Substring,  // *=
    Suffix,  // $=
}

impl AttrSelectorOperator {
    pub fn eval_str(self, element_attr_value: &str, attr_selector_value: &str,
                    case_sensitivity: CaseSensitivity) -> bool {
        let e = element_attr_value.as_bytes();
        let s = attr_selector_value.as_bytes();
        let case = case_sensitivity;
        match self {
            AttrSelectorOperator::Equal => {
                case.eq(e, s)
            }
            AttrSelectorOperator::Prefix => {
                e.len() >= s.len() && case.eq(&e[..s.len()], s)
            }
            AttrSelectorOperator::Suffix => {
                e.len() >= s.len() && case.eq(&e[(e.len() - s.len())..], s)
            }
            AttrSelectorOperator::Substring => {
                case.contains(element_attr_value, attr_selector_value)
            }
            AttrSelectorOperator::Includes => {
                element_attr_value.split(SELECTOR_WHITESPACE)
                                  .any(|part| case.eq(part.as_bytes(), s))
            }
            AttrSelectorOperator::DashMatch => {
                case.eq(e, s) || (
                    e.get(s.len()) == Some(&b'-') &&
                    case.eq(&e[..s.len()], s)
                )
            }
        }
    }
}

/// The definition of whitespace per CSS Selectors Level 3 § 4.
pub static SELECTOR_WHITESPACE: &'static [char] = &[' ', '\t', '\n', '\r', '\x0C'];

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum CaseSensitivity {
    CaseSensitive,  // Selectors spec says language-defined, but HTML says sensitive.
    AsciiCaseInsensitive,
}

impl CaseSensitivity {
    pub fn eq(self, a: &[u8], b: &[u8]) -> bool {
        match self {
            CaseSensitivity::CaseSensitive => a == b,
            CaseSensitivity::AsciiCaseInsensitive => a.eq_ignore_ascii_case(b)
        }
    }

    pub fn contains(self, haystack: &str, needle: &str) -> bool {
        match self {
            CaseSensitivity::CaseSensitive => haystack.contains(needle),
            CaseSensitivity::AsciiCaseInsensitive => {
                if let Some((&n_first_byte, n_rest)) = needle.as_bytes().split_first() {
                    haystack.bytes().enumerate().any(|(i, byte)| {
                        if !byte.eq_ignore_ascii_case(&n_first_byte) {
                            return false
                        }
                        let after_this_byte = &haystack.as_bytes()[i + 1..];
                        match after_this_byte.get(..n_rest.len()) {
                            None => false,
                            Some(haystack_slice) => {
                                haystack_slice.eq_ignore_ascii_case(n_rest)
                            }
                        }
                    })
                } else {
                    // any_str.contains("") == true,
                    // though these cases should be handled with *NeverMatches and never go here.
                    true
                }
            }
        }
    }
}
