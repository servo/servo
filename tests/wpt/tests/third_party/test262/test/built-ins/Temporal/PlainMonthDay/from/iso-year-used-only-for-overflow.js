// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: >
  As a special case when constructing PlainMonthDay with iso8601 calendar, the
  year property is only used for applying the overflow option, not a range check
features: [Temporal]
includes: [temporalHelpers.js]
---*/

var outOfRangeCommonYears = [-999999, 1000000000000005];
var outOfRangeLeapYears = [-1000000, 1000000000000004];

for (const year of outOfRangeCommonYears) {
  var result = Temporal.PlainMonthDay.from({ year, month: 1, day: 1 });
  TemporalHelpers.assertPlainMonthDay(result, "M01", 1, `ISO year ${year} is not checked for range`);

  var commonResult = Temporal.PlainMonthDay.from({
    year,
    monthCode: "M02",
    day: 29
  });
  TemporalHelpers.assertPlainMonthDay(commonResult, "M02", 28, `ISO year ${year} is used to apply overflow`);

  assert.throws(RangeError, function () {
    Temporal.PlainMonthDay.from({
      year,
      monthCode: "M02",
      day: 29
    }, { overflow: "reject" });
  }, "ISO year is used to apply overflow");
}

for (const year of outOfRangeLeapYears) {
  var leapResult = Temporal.PlainMonthDay.from({
    year,
    monthCode: "M02",
    day: 29
  });
  TemporalHelpers.assertPlainMonthDay(leapResult, "M02", 29, `ISO year ${year} is used to check leap year status`);
}
