// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.formatToParts
description: Checks the handling of invalid value arguments to Intl.RelativeTimeFormat.prototype.formatToParts().
info: |
    Intl.RelativeTimeFormat.prototype.formatToParts( value, unit )

    3. Let value be ? ToNumber(value).

    PartitionRelativeTimePattern ( relativeTimeFormat, value, unit )

    4. If isFinite(value) is false, then throw a RangeError exception.

features: [Intl.RelativeTimeFormat]
---*/

const rtf = new Intl.RelativeTimeFormat("en-US");

assert.sameValue(typeof rtf.formatToParts, "function", "formatToParts should be supported");

const values = [
  [undefined, "undefined"],
  [NaN, "NaN"],
  [Infinity, "Infinity"],
  [-Infinity, "-Infinity"],
  ["string", '"string"'],
  [{}, "empty object"],
  [{ toString() { return NaN; }, valueOf: undefined }, "object with toString"],
  [{ valueOf() { return NaN; }, toString: undefined }, "object with valueOf"],
];

for (const [value, name] of values) {
  assert.throws(RangeError, () => rtf.formatToParts(value, "second"), name);
}
