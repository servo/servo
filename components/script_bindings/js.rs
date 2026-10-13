/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CStr;

use bitflags::bitflags;
use js::context::JSContext;
use js::jsapi::{JSPROP_ENUMERATE, JSPROP_PERMANENT, JSPROP_READONLY};
use js::rust::{HandleObject, HandleValue};

bitflags! {
    /// Flags for property definitions in JavaScript.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct PropertyFlags: u8 {
        /// The property shows up when enumerating properties on the object.
        const ENUMERABLE = 1 << 0;
        /// The property can be changed.
        const WRITABLE = 1 << 1;
        /// The property can be deleted or have its attributes changed.
        const CONFIGURABLE = 1 << 2;
    }
}

impl PropertyFlags {
    /// Convert the flags to the underlying SpiderMonkey `JSPROP_*` attributes.
    pub fn as_jsprop_flags(&self) -> u32 {
        let mut attrs: u32 = 0;
        if self.contains(Self::ENUMERABLE) {
            attrs |= JSPROP_ENUMERATE as u32;
        }
        if !self.contains(Self::CONFIGURABLE) {
            attrs |= JSPROP_PERMANENT as u32;
        }
        if !self.contains(Self::WRITABLE) {
            attrs |= JSPROP_READONLY as u32;
        }
        attrs
    }
}

/// Safe wrapper around [`js::rust::wrappers2::JS_DefineProperty`].
#[allow(non_snake_case)]
#[inline]
pub fn JS_DefineProperty(
    cx: &mut JSContext,
    obj: HandleObject,
    name: &CStr,
    value: HandleValue,
    flags: PropertyFlags,
) -> bool {
    let attrs = flags.as_jsprop_flags();
    unsafe { js::rust::wrappers2::JS_DefineProperty(cx, obj, name.as_ptr(), value, attrs) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_flags_mapping() {
        // Enumerable, Writable, Configurable -> only JSPROP_ENUMERATE
        let all = PropertyFlags::ENUMERABLE | PropertyFlags::WRITABLE | PropertyFlags::CONFIGURABLE;
        assert_eq!(all.as_jsprop_flags(), JSPROP_ENUMERATE as u32);

        // Constant property: Enumerable, but not Writable or Configurable
        // -> JSPROP_ENUMERATE | JSPROP_READONLY | JSPROP_PERMANENT
        let constant = PropertyFlags::ENUMERABLE;
        assert_eq!(
            constant.as_jsprop_flags(),
            (JSPROP_ENUMERATE | JSPROP_READONLY | JSPROP_PERMANENT) as u32
        );

        // Non-enumerable, non-writable, non-configurable (empty)
        // -> JSPROP_READONLY | JSPROP_PERMANENT
        let empty = PropertyFlags::empty();
        assert_eq!(
            empty.as_jsprop_flags(),
            (JSPROP_READONLY | JSPROP_PERMANENT) as u32
        );

        // Writable and Configurable, not Enumerable
        // -> 0
        let method = PropertyFlags::WRITABLE | PropertyFlags::CONFIGURABLE;
        assert_eq!(method.as_jsprop_flags(), 0);
    }
}
