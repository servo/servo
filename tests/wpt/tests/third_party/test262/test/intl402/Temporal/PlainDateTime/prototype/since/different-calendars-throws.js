// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Fail if the argument is a PlainDateTime with a different calendar
features: [Temporal]
---*/

const dt1 = new Temporal.PlainDateTime(2000, 1, 1, 0, 0, 0, 0, 0, 0);
const dt2 = new Temporal.PlainDateTime(2000, 1, 1, 0, 0, 0, 0, 0, 0, "gregory");

assert.throws(
  RangeError,
  () => dt1.since(dt2),
  "different calendars not allowed"
);
