// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Check constraining days when year changes (islamic-civil calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "islamic-civil";
const options = { overflow: "reject" };

// Month 12 (Dhu al-Hijjah) has 29 days in common years and 30 in leap years.
// AH 1442 and 1445 are leap years.

const leapDay = Temporal.ZonedDateTime.from({ year: 1445, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 1442 }, options).toPlainDateTime(),
  1442, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "day not constrained when moving to another leap year",
  "ah", 1442);

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 1444 }).toPlainDateTime(),
  1444, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "day constrained when moving to a common year",
  "ah", 1444);

assert.throws(RangeError, function () {
  leapDay.with({ year: 1444 }, options);
}, "reject when moving to a common year");
