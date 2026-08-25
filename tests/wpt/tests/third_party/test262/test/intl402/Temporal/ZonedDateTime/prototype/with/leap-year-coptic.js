// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Check various basic calculations involving leap years (coptic calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "coptic";
const options = { overflow: "reject" };

const leapDay = Temporal.ZonedDateTime.from({ year: 1739, monthCode: "M13", day: 6, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 1740 }).toPlainDateTime(),
  1740, 13, "M13", 5, 12, 34, 0, 0, 0, 0, "Changing year on leap day to common year constrains to day 5 of epagomenal month",
  "am", 1740);
assert.throws(RangeError, function () {
  leapDay.with({ year: 1740 }, options);
}, "Changing year on leap day to common year rejects");

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 1735 }, options).toPlainDateTime(),
  1735, 13, "M13", 6, 12, 34, 0, 0, 0, 0, "Changing year on leap day to another leap year constrains to day 6 of epagomenal month",
  "am", 1735);
