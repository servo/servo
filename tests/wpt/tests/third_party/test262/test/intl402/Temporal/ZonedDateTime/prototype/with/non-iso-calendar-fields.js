// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Properties passed to with() are calendar fields, not ISO date
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC", "hebrew");

const resultYear = instance.with({ year: 5762 });
assert.sameValue(resultYear.year, 5762, "year is changed");
assert.sameValue(resultYear.month, 12, "month is not changed");
assert.sameValue(resultYear.monthCode, "M12", "month code is not changed");
assert.sameValue(resultYear.day, 21, "day is not changed");

const resultMonth = instance.with({ month: 11 });
assert.sameValue(resultMonth.year, 5761, "year is not changed");
assert.sameValue(resultMonth.month, 11, "month is changed");
assert.sameValue(resultMonth.monthCode, "M11", "month code is changed");
assert.sameValue(resultMonth.day, 21, "day is not changed");

const resultMonthCode = instance.with({ monthCode: "M11" });
assert.sameValue(resultMonthCode.year, 5761, "year is not changed");
assert.sameValue(resultMonthCode.month, 11, "month is changed");
assert.sameValue(resultMonthCode.monthCode, "M11", "month code is changed");
assert.sameValue(resultMonthCode.day, 21, "day is not changed");

const resultDay = instance.with({ day: 24 });
assert.sameValue(resultDay.year, 5761, "year is not changed");
assert.sameValue(resultDay.month, 12, "month is not changed");
assert.sameValue(resultDay.monthCode, "M12", "month code is not changed");
assert.sameValue(resultDay.day, 24, "day is changed");
