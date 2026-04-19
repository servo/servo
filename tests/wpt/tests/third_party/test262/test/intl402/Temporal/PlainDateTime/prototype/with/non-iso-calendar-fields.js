// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Properties passed to with() are calendar fields, not ISO date
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2024, 8, 8, 12, 34, 56, 987, 654, 321, "hebrew");

const resultYear = instance.with({ year: 5783 });
assert.sameValue(resultYear.year, 5783, "year is changed");
assert.sameValue(resultYear.month, 11, "month is changed because year has different number of months");
assert.sameValue(resultYear.monthCode, "M11", "month code is not changed");
assert.sameValue(resultYear.day, 4, "day is not changed");

const resultMonth = instance.with({ month: 13 });
assert.sameValue(resultMonth.year, 5784, "year is not changed");
assert.sameValue(resultMonth.month, 13, "month is changed");
assert.sameValue(resultMonth.monthCode, "M12", "month code is changed");
assert.sameValue(resultMonth.day, 4, "day is not changed");

const resultMonthCode = instance.with({ monthCode: "M10" });
assert.sameValue(resultMonthCode.year, 5784, "year is not changed");
assert.sameValue(resultMonthCode.month, 11, "month is changed");
assert.sameValue(resultMonthCode.monthCode, "M10", "month code is changed");
assert.sameValue(resultMonthCode.day, 4, "day is not changed");

const resultDay = instance.with({ day: 24 });
assert.sameValue(resultDay.year, 5784, "year is not changed");
assert.sameValue(resultDay.month, 12, "month is not changed");
assert.sameValue(resultDay.monthCode, "M11", "month code is not changed");
assert.sameValue(resultDay.day, 24, "day is changed");
