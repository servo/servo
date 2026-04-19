// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.with
description: Basic tests for with().
includes: [compareArray.js, temporalHelpers.js]
features: [Symbol, Temporal]
---*/

const md = Temporal.PlainMonthDay.from("01-15");

TemporalHelpers.assertPlainMonthDay(md.with({ day: 22 }),
  "M01", 22, "with({day})");

TemporalHelpers.assertPlainMonthDay(md.with({ month: 12 }),
  "M12", 15, "with({month})");

TemporalHelpers.assertPlainMonthDay(md.with({ monthCode: "M12" }),
  "M12", 15, "with({monthCode})");

TemporalHelpers.assertPlainMonthDay(md.with({ month: 12, monthCode: "M12" }),
  "M12", 15, "with({month, monthCode}) agree");

assert.throws(RangeError, () => md.with({ month: 12, monthCode: "M11" }), "with({month, monthCode}) disagree");

TemporalHelpers.assertPlainMonthDay(md.with({ year: 2000, month: 12 }),
  "M12", 15, "with({year, month})");

TemporalHelpers.assertPlainMonthDay(md.with({ year: 2000 }),
  "M01", 15, "with({year})");

assert.throws(TypeError, () => md.with({ day: 1, calendar: "iso8601" }), "with({calendar})");

assert.throws(TypeError, () => md.with({ day: 1, timeZone: "UTC" }), "with({timeZone})");

assert.throws(TypeError, () => md.with({}), "with({})");
assert.throws(TypeError, () => md.with({ months: 12 }), "with({months})");

TemporalHelpers.assertPlainMonthDay(md.with({ monthCode: "M12", days: 1 }),
  "M12", 15, "with({monthCode, days})");
