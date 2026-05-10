// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Check various basic calculations involving leap years (ethiopic calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

const leapDay = Temporal.ZonedDateTime.from({ year: 2015, monthCode: "M13", day: 6, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 2016 }).toPlainDateTime(),
  2016, 13, "M13", 5, 12, 34, 0, 0, 0, 0, "Changing year on leap day to a common year constrains to day 5 of epagomenal month",
  "am", 2016);
assert.throws(RangeError, function () {
  leapDay.with({ year: 2016 }, options);
}, "Changing year on leap day to a common year rejects");

TemporalHelpers.assertPlainDateTime(
  leapDay.with({ year: 2011 }, options).toPlainDateTime(),
  2011, 13, "M13", 6, 12, 34, 0, 0, 0, 0, "Changing year on leap day to another leap year constrains to day 6 of epagomenal month",
  "am", 2011);
