// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Check constraining days when year changes (gregory calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "gregory";
const options = { overflow: "reject" };

const leapDay = Temporal.PlainDate.from({ year: 2016, monthCode: "M02", day: 29, calendar }, options);

TemporalHelpers.assertPlainDate(
  leapDay.with({ year: 2012 }, options),
  2012, 2, "M02", 29, "day not constrained when moving to another leap year",
  "ce", 2012);

TemporalHelpers.assertPlainDate(
  leapDay.with({ year: 2018 }),
  2018, 2, "M02", 28, "day constrained when moving to a common year",
  "ce", 2018);

assert.throws(RangeError, function () {
  leapDay.with({ year: 2018 }, options);
}, "reject when moving to a common year");
