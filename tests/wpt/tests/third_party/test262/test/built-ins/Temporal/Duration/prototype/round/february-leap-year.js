// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: RangeError thrown when largestUnit option not one of the allowed string values
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Based on a test case by André Bargull <andre.bargull@gmail.com>

// Note: One day after February in a leap year.
const relativeTo = new Temporal.PlainDate(1972, 3, 1);

const options = {
  largestUnit: "years",
  relativeTo,
};

const twoDaysLessThanFourYears = new Temporal.Duration(3, 11, 0, 27);
TemporalHelpers.assertDuration(
  twoDaysLessThanFourYears.round(options),
  3, 11, 0, 27, 0, 0, 0, 0, 0, 0,
  "Two days less than four years starting in February in a leap year shouldn't balance up"
);

const oneDayLessThanFourYears = new Temporal.Duration(3, 11, 0, 28);
TemporalHelpers.assertDuration(
  oneDayLessThanFourYears.round(options),
  3, 11, 0, 28, 0, 0, 0, 0, 0, 0,
  "One day less than four years starting in February in a leap year shouldn't balance up"
);

const fourYears = new Temporal.Duration(3, 11, 0, 29);
TemporalHelpers.assertDuration(
  fourYears.round(options),
  4, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "Four years starting in February in a leap year should balance up"
);
