// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.formatToParts
description: Checks the handling of plural unit arguments to Intl.RelativeTimeFormat.prototype.formatToParts().
info: |
    FormatRelativeTimeToParts ( relativeTimeFormat, value, unit )

    3. Let n be 0.
    4. For each part in parts, do:
        a. Let O be ObjectCreate(%ObjectPrototype%).
        b. Perform ! CreateDataPropertyOrThrow(O, "type", part.[[Type]]).
        c. Perform ! CreateDataPropertyOrThrow(O, "value", part.[[Value]]).
        d. If part has a [[Unit]] field,
            i. Perform ! CreateDataPropertyOrThrow(O, "unit", part.[[Unit]]).
        e. Perform ! CreateDataPropertyOrThrow(result, ! ToString(n), O).
        f. Increment n by 1.

features: [Intl.RelativeTimeFormat]
includes: [propertyHelper.js]
---*/

const rtf = new Intl.RelativeTimeFormat("en-US");

assert.sameValue(typeof rtf.formatToParts, "function", "formatToParts should be supported");

const parts = rtf.formatToParts(3, "second");

assert.sameValue(Object.getPrototypeOf(parts), Array.prototype, "parts: prototype");
assert.sameValue(Array.isArray(parts), true, "parts: isArray");
assert.sameValue(parts.length, 3, "parts: length");

assert.sameValue(Object.getPrototypeOf(parts[0]), Object.prototype, "parts[0]: prototype");
verifyProperty(parts[0], "type", {
  value: "literal",
  writable: true,
  enumerable: true,
  configurable: true,
});
verifyProperty(parts[0], "value", {
  value: "in ",
  writable: true,
  enumerable: true,
  configurable: true,
});


assert.sameValue(Object.getPrototypeOf(parts[1]), Object.prototype, "parts[1]: prototype");
verifyProperty(parts[1], "type", {
  value: "integer",
  writable: true,
  enumerable: true,
  configurable: true,
});
verifyProperty(parts[1], "value", {
  value: "3",
  writable: true,
  enumerable: true,
  configurable: true,
});
verifyProperty(parts[1], "unit", {
  value: "second",
  writable: true,
  enumerable: true,
  configurable: true,
});


assert.sameValue(Object.getPrototypeOf(parts[2]), Object.prototype, "parts[2]: prototype");
verifyProperty(parts[2], "type", {
  value: "literal",
  writable: true,
  enumerable: true,
  configurable: true,
});
verifyProperty(parts[2], "value", {
  value: " seconds",
  writable: true,
  enumerable: true,
  configurable: true,
});
