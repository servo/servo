// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Reject value for overflow option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const propertyBag = { year: 2000, month: 13 };
const plainYearMonth = Temporal.PlainYearMonth.from(propertyBag, { overflow: "constrain" });
TemporalHelpers.assertPlainYearMonth(plainYearMonth, 2000, 12, "M12", "default overflow is constrain");

const opt = { overflow: "constrain" };

let result = Temporal.PlainYearMonth.from({ year: 2021, month: 1 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 1, "M01", "month 1 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 2 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 2, "M02", "month 2 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 3 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 3, "M03", "month 3 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 4 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 4, "M04", "month 4 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 5 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 5, "M05", "month 5 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 6 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 6, "M06", "month 6 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 7 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 7, "M07", "month 7 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 8 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 8, "M08", "month 8 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 9 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 9, "M09", "month 9 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 10 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 10, "M10", "month 10 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 11 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 11, "M11", "month 11 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 12 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 12, "M12", "month 12 with overflow constrain");

assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ year: 2021, month: -99999 }, opt),
  "negative month -99999 is out of range even with overflow constrain"
)
assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ year: 2021, month: -1 }, opt),
  "negative month -1 is out of range even with overflow constrain"
)
assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ year: 2021, month: 0 }, opt),
  "month zero is out of range even with overflow constrain"
)

result = Temporal.PlainYearMonth.from({ year: 2021, month: 13 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 12, "M12", "month 13 is constrained to 12");
result = Temporal.PlainYearMonth.from({ year: 2021, month: 99999 }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 12, "M12", "month 99999 is constrained to 12");

result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M01" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 1, "M01", "monthCode M01 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M02" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 2, "M02", "monthCode M02 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M03" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 3, "M03", "monthCode M03 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M04" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 4, "M04", "monthCode M04 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M05" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 5, "M05", "monthCode M05 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M06" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 6, "M06", "monthCode M06 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M07" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 7, "M07", "monthCode M07 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M08" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 8, "M08", "monthCode M08 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M09" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 9, "M09", "monthCode M09 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M10" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 10, "M10", "monthCode M10 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M11" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 11, "M11", "monthCode M11 with overflow constrain");
result = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M12" }, opt);
TemporalHelpers.assertPlainYearMonth(result, 2021, 12, "M12", "monthCode M12 with overflow constrain");
