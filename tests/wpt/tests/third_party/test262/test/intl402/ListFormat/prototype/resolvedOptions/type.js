// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.resolvedOptions
description: Checks the properties of the result of Intl.ListFormat.prototype.resolvedOptions().
info: |
    Intl.ListFormat.prototype.resolvedOptions ()

    4. Let options be ! ObjectCreate(%ObjectPrototype%).
    5. For each row of Table 1, except the header row, do
        d. Perform ! CreateDataPropertyOrThrow(options, p, v).
includes: [propertyHelper.js]
features: [Intl.ListFormat]
---*/

const lf = new Intl.ListFormat("en-us", { "style": "short", "type": "unit" });
const options = lf.resolvedOptions();
assert.sameValue(Object.getPrototypeOf(options), Object.prototype, "Prototype");

verifyProperty(options, "locale", {
  value: "en-US",
  writable: true,
  enumerable: true,
  configurable: true,
});

verifyProperty(options, "type", {
  value: "unit",
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
