// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Check various basic calculations involving leap years (indian calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "indian";
const options = { overflow: "reject" };

const leapDay = Temporal.PlainDateTime.from({ year: 1946, monthCode: "M01", day: 31, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 1947 }),
  1947, 1, "M01", 30, 12, 34, 0, 0, 0, 0, "Changing year to a common year on leap day constrains to 30 Chaitra",
  "shaka", 1947);
assert.throws(RangeError, function () {
  leapDay.with({ year: 1947 }, options);
}, "Changing year to a common year on leap day rejects");

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 1942 }, options),
  1942, 1, "M01", 31, 12, 34, 0, 0, 0, 0, "Changing year to another leap year on leap day does not reject",
  "shaka", 1942);

const nonLeapDayInLeapYear = Temporal.PlainDateTime.from({ year: 1926, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  nonLeapDayInLeapYear.with({ day: 31 }, options),
  1926, 1, "M01", 31, 12, 34, 0, 0, 0, 0, "Changing non-leap day to leap day in a leap year does not reject",
  "shaka", 1926);

const commonYear = Temporal.PlainDateTime.from({ year: 1927, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  commonYear.with({ day: 31 }),
  1927, 1, "M01", 30, 12, 34, 0, 0, 0, 0, "Changing day to leap day in a common year constrains to 30 Chaitra",
  "shaka", 1927);
assert.throws(RangeError, function () {
  commonYear.with({ day: 31 }, options);
}, "Changing day to leap day in a common year rejects");
