// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: The month and month code should agree
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);

assert.throws(
  RangeError,
  () => datetime.with({ month: 5, monthCode: "M06" }),
  "month and monthCode must agree"
);
