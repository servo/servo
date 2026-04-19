// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Using different calendars is not acceptable
features: [Temporal]
---*/

const dt1 = new Temporal.PlainDateTime(2000, 1, 1, 0, 0, 0, 0, 0, 0);
const dt2 = new Temporal.PlainDateTime(2000, 1, 1, 0, 0, 0, 0, 0, 0, "gregory");

assert.throws(
  RangeError,
  () => dt1.until(dt2),
  "cannot use until with PDTs having different calendars"
);
