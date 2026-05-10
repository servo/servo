// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: RangeError thrown if a date-only string is passed in a PlainTime context
features: [Temporal, arrow-function]
---*/

const arg = "2019-10-01";
const instance = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);
assert.throws(
  RangeError,
  () => instance.since(arg),
  "Date-only string throws, does not implicitly convert to midnight"
);
