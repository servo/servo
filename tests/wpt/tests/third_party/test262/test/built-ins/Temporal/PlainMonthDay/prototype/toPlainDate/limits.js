// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.toplaindate
description: Throws a RangeError if the resulting PlainDate is out of range
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const jan1 = Temporal.PlainMonthDay.from("01-01");
const dec31 = Temporal.PlainMonthDay.from("12-31");

const minYear = -271821;
assert.throws(RangeError, () => jan1.toPlainDate({ year: minYear }), "jan1 min");
const apr18 = Temporal.PlainMonthDay.from("04-18");
assert.throws(RangeError, () => apr18.toPlainDate({ year: minYear }), "apr18 min");
TemporalHelpers.assertPlainDate(Temporal.PlainMonthDay.from("04-19").toPlainDate({ year: minYear }),
  minYear, 4, "M04", 19, "apr19 min");
TemporalHelpers.assertPlainDate(jan1.toPlainDate({ year: minYear + 1 }),
  minYear + 1, 1, "M01", 1, "jan1 min");

const maxYear = 275760;
assert.throws(RangeError, () => dec31.toPlainDate({ year: maxYear }), "dec31 max");
const sep14 = Temporal.PlainMonthDay.from("09-14");
assert.throws(RangeError, () => sep14.toPlainDate({ year: maxYear }), "sep14 max");
TemporalHelpers.assertPlainDate(Temporal.PlainMonthDay.from("09-13").toPlainDate({ year: maxYear }),
  maxYear, 9, "M09", 13, "max");
TemporalHelpers.assertPlainDate(dec31.toPlainDate({ year: maxYear - 1 }),
  maxYear - 1, 12, "M12", 31, "dec31 max");
