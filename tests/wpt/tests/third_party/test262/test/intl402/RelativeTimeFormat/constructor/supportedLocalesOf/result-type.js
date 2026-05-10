// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.supportedLocalesOf
description: Verifies the type of the return value of Intl.RelativeTimeFormat.supportedLocalesOf().
info: |
    Intl.RelativeTimeFormat.supportedLocalesOf ( locales [, options ])
includes: [propertyHelper.js]
features: [Intl.RelativeTimeFormat]
---*/

const result = Intl.RelativeTimeFormat.supportedLocalesOf("en");
assert.sameValue(Array.isArray(result), true,
  "Array.isArray() should return true");
assert.sameValue(Object.getPrototypeOf(result), Array.prototype,
  "The prototype should be Array.prototype");
assert.sameValue(Object.isExtensible(result), true,
  "Object.isExtensible() should return true");

assert.notSameValue(result.length, 0);
for (let i = 0; i < result.length; ++i) {
  verifyProperty(result, String(i), {
    "writable": true,
    "enumerable": true,
    "configurable": true,
  });
}

verifyProperty(result, "length", {
  "enumerable": false,
  "configurable": false,
});
