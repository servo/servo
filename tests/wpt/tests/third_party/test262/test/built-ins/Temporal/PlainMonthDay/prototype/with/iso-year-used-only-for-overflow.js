// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.with
description: >
  As a special case applying partial fields to PlainMonthDay with iso8601
  calendar, the year property is only used for applying the overflow option, not
  a range check
features: [Temporal]
includes: [temporalHelpers.js]
---*/

var outOfRangeCommonYear = -999999;
var outOfRangeLeapYear = -1000000;

var md = new Temporal.PlainMonthDay(1, 1, "iso8601", 1972);
var result = md.with({ year: outOfRangeCommonYear });
TemporalHelpers.assertPlainMonthDay(result, "M01", 1, "ISO year is not checked for range");

var leap = new Temporal.PlainMonthDay(2, 29, "iso8601", 1972);
var commonResult = leap.with({ year: outOfRangeCommonYear });
TemporalHelpers.assertPlainMonthDay(commonResult, "M02", 28, "ISO year is used to apply overflow");

assert.throws(RangeError, function () {
  leap.with({ year: outOfRangeCommonYear }, { overflow: "reject" });
}, "ISO year is used to apply overflow");

var leapResult = leap.with({ year: outOfRangeLeapYear });
TemporalHelpers.assertPlainMonthDay(leapResult, "M02", 29, "ISO year is used to apply overflow");
