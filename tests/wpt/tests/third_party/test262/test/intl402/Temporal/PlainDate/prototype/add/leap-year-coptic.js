// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Check various basic calculations involving leap years (coptic calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "coptic";
const options = { overflow: "reject" };

const leapDay = Temporal.PlainDate.from({ year: 1739, monthCode: "M13", day: 6, calendar }, options);

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);

TemporalHelpers.assertPlainDate(
  leapDay.add(years1),
  1740, 13, "M13", 5, "Adding 1 year to leap day constrains to day 5 of epagomenal month",
  "am", 1740);
assert.throws(RangeError, function () {
  leapDay.add(years1, options);
}, "Adding 1 year to leap day rejects");

TemporalHelpers.assertPlainDate(
  leapDay.add(years1n),
  1738, 13, "M13", 5, "Subtracting 1 year from leap day constrains to day 5 of epagomenal month",
  "am", 1738);
assert.throws(RangeError, function () {
  leapDay.add(years1n, options);
}, "Subtracting 1 year from leap day rejects");

TemporalHelpers.assertPlainDate(
  leapDay.add(years4, options),
  1743, 13, "M13", 6, "Adding 4 years to leap day goes to the next leap day",
  "am", 1743);

TemporalHelpers.assertPlainDate(
  leapDay.add(years4n, options),
  1735, 13, "M13", 6, "Subtracting 4 years from leap day goes to the previous leap day",
  "am", 1735);
