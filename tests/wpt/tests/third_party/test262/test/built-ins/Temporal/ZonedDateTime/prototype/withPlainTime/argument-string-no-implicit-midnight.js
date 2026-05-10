// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: RangeError thrown if a date-only string is passed in a PlainTime context
features: [Temporal, arrow-function]
---*/

const arg = "2019-10-01";
const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");
assert.throws(
  RangeError,
  () => instance.withPlainTime(arg),
  "Date-only string throws, does not implicitly convert to midnight"
);
