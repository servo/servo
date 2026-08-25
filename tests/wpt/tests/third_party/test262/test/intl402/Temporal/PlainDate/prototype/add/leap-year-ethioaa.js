// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Check various basic calculations involving leap years (ethioaa calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethioaa";
const options = { overflow: "reject" };

const leapDay = Temporal.PlainDate.from({ year: 7515, monthCode: "M13", day: 6, calendar }, options);

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years4 = new Temporal.Duration(4);
const years4n = new Temporal.Duration(-4);

TemporalHelpers.assertPlainDate(
  leapDay.add(years1),
  7516, 13, "M13", 5, "Adding 1 year to leap day constrains to day 5 of epagomenal month",
  "aa", 7516);
assert.throws(RangeError, function () {
  leapDay.add(years1, options);
}, "Adding 1 year to leap day rejects");

TemporalHelpers.assertPlainDate(
  leapDay.add(years1n),
  7514, 13, "M13", 5, "Subtracting 1 year from leap day constrains to day 5 of epagomenal month",
  "aa", 7514);
assert.throws(RangeError, function () {
  leapDay.add(years1n, options);
}, "Subtracting 1 year from leap day rejects");

TemporalHelpers.assertPlainDate(
  leapDay.add(years4, options),
  7519, 13, "M13", 6, "Adding 4 years to leap day goes to the next leap day",
  "aa", 7519);

TemporalHelpers.assertPlainDate(
  leapDay.add(years4n, options),
  7511, 13, "M13", 6, "Subtracting 4 years from leap day goes to the previous leap day",
  "aa", 7511);
