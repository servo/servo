// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
    The relativeTo option is required when the Duration contains years, months,
    or weeks, and unit is days; or unit is weeks or months
features: [Temporal, arrow-function]
---*/

const oneYear = new Temporal.Duration(1);
const oneMonth = new Temporal.Duration(0, 1);
const oneWeek = new Temporal.Duration(0, 0, 1);
const oneDay = new Temporal.Duration(0, 0, 0, 1);

const options = { unit: "days" };
assert.sameValue(oneDay.total(options), 1, "days do not require relativeTo");
assert.sameValue(oneDay.total("days"), 1, "days do not require relativeTo (string shorthand)");
assert.throws(RangeError, () => oneWeek.total(options), "total days of weeks requires relativeTo");
assert.throws(RangeError, () => oneWeek.total("days"), "total days of weeks requires relativeTo (string shorthand)");
assert.throws(RangeError, () => oneMonth.total(options), "total days of months requires relativeTo");
assert.throws(RangeError, () => oneMonth.total("days"), "total days of months requires relativeTo (string shorthand)");
assert.throws(RangeError, () => oneYear.total(options), "total days of years requires relativeTo");
assert.throws(RangeError, () => oneYear.total("days"), "total days of years requires relativeTo (string shorthand)");

["months", "weeks"].forEach((unit) => {
  [oneDay, oneWeek, oneMonth, oneYear].forEach((duration) => {
    assert.throws(RangeError, () => duration.total({ unit }), `${duration} total ${unit} requires relativeTo`);
    assert.throws(RangeError, () => duration.total(unit), `${duration} total ${unit} requires relativeTo (string shorthand)`);
  });
});
