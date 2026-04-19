// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Properties passed to with() are calendar fields, not ISO date
features: [Temporal]
---*/

const instance = Temporal.PlainYearMonth.from({ calendar: "hebrew", year: 5784, monthCode: "M11" });

const resultYear = instance.with({ year: 5783 });
assert.sameValue(resultYear.year, 5783, "year is changed");
assert.sameValue(resultYear.month, 11, "month is changed because year has different number of months");
assert.sameValue(resultYear.monthCode, "M11", "month code is not changed");

const resultMonth = instance.with({ month: 13 });
assert.sameValue(resultMonth.year, 5784, "year is not changed");
assert.sameValue(resultMonth.month, 13, "month is changed");
assert.sameValue(resultMonth.monthCode, "M12", "month code is changed");

const resultMonthCode = instance.with({ monthCode: "M10" });
assert.sameValue(resultMonthCode.year, 5784, "year is not changed");
assert.sameValue(resultMonthCode.month, 11, "month is changed");
assert.sameValue(resultMonthCode.monthCode, "M10", "month code is changed");
