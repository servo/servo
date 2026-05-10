// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Check various basic calculations involving leap years (ethioaa calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethioaa";
const options = { overflow: "reject" };

const leapDay = Temporal.PlainDateTime.from({ year: 7515, monthCode: "M13", day: 6, hour: 12, minute: 34, calendar }, options);

// Years

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 7516 }),
  7516, 13, "M13", 5, 12, 34, 0, 0, 0, 0, "Changing year on leap day to common year constrains to day 5 of epagomenal month",
  "aa", 7516);
assert.throws(RangeError, function () {
  leapDay.with({ year: 7516 }, options);
}, "Changing year on leap day to common year rejects");

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 7511 }, options),
  7511, 13, "M13", 6, 12, 34, 0, 0, 0, 0, "Changing year on leap day to another leap year constrains to day 6 of epagomenal month",
  "aa", 7511);
