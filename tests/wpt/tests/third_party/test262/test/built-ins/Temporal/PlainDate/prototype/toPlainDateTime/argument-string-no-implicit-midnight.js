// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: RangeError thrown if a date-only string is passed in a PlainTime context
features: [Temporal, arrow-function]
---*/

const arg = "2019-10-01";
const instance = new Temporal.PlainDate(2000, 5, 2);
assert.throws(
  RangeError,
  () => instance.toPlainDateTime(arg),
  "Date-only string throws, does not implicitly convert to midnight"
);
