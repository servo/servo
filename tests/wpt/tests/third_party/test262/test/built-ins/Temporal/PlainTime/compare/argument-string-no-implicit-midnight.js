// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: RangeError thrown if a date-only string is passed in a PlainTime context
features: [Temporal, arrow-function]
---*/

const arg = "2019-10-01";
const midnight = new Temporal.PlainTime();
assert.throws(
  RangeError,
  () => Temporal.PlainTime.compare(arg, midnight),
  "Date-only string throws, does not implicitly convert to midnight (first argument)"
);
assert.throws(
  RangeError,
  () => Temporal.PlainTime.compare(midnight, arg),
  "Date-only string throws, does not implicitly convert to midnight (second argument)"
);
