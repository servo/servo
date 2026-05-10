// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
    The relativeTo option is required when the Duration contains years, months,
    or weeks, and largestUnit is days; or largestUnit is weeks or months
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const oneYear = new Temporal.Duration(1);
const oneMonth = new Temporal.Duration(0, 1);
const oneWeek = new Temporal.Duration(0, 0, 1);
const oneDay = new Temporal.Duration(0, 0, 0, 1);

const options = { largestUnit: "days" };
TemporalHelpers.assertDuration(oneDay.round(options), 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, "days do not require relativeTo");
assert.throws(RangeError, () => oneWeek.round(options), "balancing weeks to days requires relativeTo");
assert.throws(RangeError, () => oneMonth.round(options), "balancing months to days requires relativeTo");
assert.throws(RangeError, () => oneYear.round(options), "balancing years to days requires relativeTo");

["months", "weeks"].forEach((largestUnit) => {
  [oneDay, oneWeek, oneMonth, oneYear].forEach((duration) => {
    assert.throws(RangeError, () => duration.round({ largestUnit }), `balancing ${duration} to ${largestUnit} requires relativeTo`);
  });
});
