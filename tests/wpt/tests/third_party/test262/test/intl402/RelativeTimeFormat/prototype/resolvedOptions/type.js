// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.resolvedOptions
description: Checks the properties of the result of Intl.RelativeTimeFormat.prototype.resolvedOptions().
info: |
    Intl.RelativeTimeFormat.prototype.resolvedOptions ()

    4. Let options be ! ObjectCreate(%ObjectPrototype%).
    5. For each row of Table 1, except the header row, do
        d. Perform ! CreateDataPropertyOrThrow(options, p, v).
includes: [propertyHelper.js]
features: [Intl.RelativeTimeFormat]
---*/

const rtf = new Intl.RelativeTimeFormat("en-us", { "style": "short", "numeric": "auto" });
const options = rtf.resolvedOptions();
assert.sameValue(Object.getPrototypeOf(options), Object.prototype, "Prototype");

verifyProperty(options, "locale", {
  value: "en-US",
  writable: true,
  enumerable: true,
  configurable: true,
});

verifyProperty(options, "style", {
  value: "short",
  writable: true,
  enumerable: true,
  configurable: true,
});

verifyProperty(options, "numeric", {
  value: "auto",
  writable: true,
  enumerable: true,
  configurable: true,
});
