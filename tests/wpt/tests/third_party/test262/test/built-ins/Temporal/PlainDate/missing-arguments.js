// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate
description: RangeError thrown when constructor invoked with no argument
includes: [compareArray.js]
features: [Temporal]
---*/

const expected = [
  "valueOf year",
  "valueOf month",
];
const actual = [];
const args = [
  { valueOf() { actual.push("valueOf year"); return 1; } },
  { valueOf() { actual.push("valueOf month"); return 1; } },
];

assert.throws(RangeError, () => new Temporal.PlainDate(...args));
assert.compareArray(actual, expected, "order of operations");

assert.throws(RangeError, () => new Temporal.PlainDate(), "no arguments");
assert.throws(RangeError, () => new Temporal.PlainDate(2021), "only year");

