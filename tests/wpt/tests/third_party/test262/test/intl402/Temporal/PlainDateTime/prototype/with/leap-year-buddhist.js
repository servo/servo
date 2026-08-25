// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Check constraining days when year changes (buddhist calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "buddhist";
const options = { overflow: "reject" };

const leapDay = Temporal.PlainDateTime.from({ year: 2559, monthCode: "M02", day: 29, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 2555 }, options),
  2555, 2, "M02", 29,  12, 34, 0, 0, 0, 0,"day not constrained when moving to another leap year",
  "be", 2555);

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 2561 }),
  2561, 2, "M02", 28,  12, 34, 0, 0, 0, 0,"day constrained when moving to a common year",
  "be", 2561);

assert.throws(RangeError, function () {
  leapDay.with({ year: 2561 }, options);
}, "reject when moving to a common year");
