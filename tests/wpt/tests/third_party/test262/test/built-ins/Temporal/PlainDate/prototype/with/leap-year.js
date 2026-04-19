// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Check constraining days when year changes
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const leapDay = new Temporal.PlainDate(2016, 2, 29);
const options = { overflow: "reject" };

TemporalHelpers.assertPlainDate(
  leapDay.with({ year: 2012 }, options),
  2012, 2, "M02", 29, "day not constrained when moving to another leap year");

TemporalHelpers.assertPlainDate(
  leapDay.with({ year: 2018 }),
  2018, 2, "M02", 28, "day constrained when moving to a common year");

assert.throws(RangeError, function () {
  leapDay.with({ year: 2018 }, options);
}, "reject when moving to a common year");
