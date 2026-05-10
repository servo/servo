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

var outOfRangeCommonYear = -999999;
var outOfRangeLeapYear = -1000000;

var result = Temporal.PlainMonthDay.from({
  year: outOfRangeCommonYear,
  month: 1,
  day: 1,
});
TemporalHelpers.assertPlainMonthDay(result, "M01", 1, "ISO year is not checked for range");

var commonResult = Temporal.PlainMonthDay.from({
  year: outOfRangeCommonYear,
  monthCode: "M02",
  day: 29
});
TemporalHelpers.assertPlainMonthDay(commonResult, "M02", 28, "ISO year is used to apply overflow");

assert.throws(RangeError, function () {
  Temporal.PlainMonthDay.from({
    year: outOfRangeCommonYear,
    monthCode: "M02",
    day: 29
  }, { overflow: "reject" });
}, "ISO year is used to apply overflow");

var leapResult = Temporal.PlainMonthDay.from({
  year: outOfRangeLeapYear,
  monthCode: "M02",
  day: 29
});
TemporalHelpers.assertPlainMonthDay(leapResult, "M02", 29, "ISO year is used to apply overflow");
