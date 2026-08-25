// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.with
description: Properties passed to with() are calendar fields, not ISO date
features: [Temporal]
---*/

const instance = Temporal.PlainMonthDay.from({ calendar: "hebrew", monthCode: "M11", day: 4 });

const resultMonthCode = instance.with({ monthCode: "M10" });
assert.sameValue(resultMonthCode.monthCode, "M10", "month code is changed");
assert.sameValue(resultMonthCode.day, 4, "day is not changed");

const resultDay = instance.with({ day: 24 });
assert.sameValue(resultDay.monthCode, "M11", "month code is not changed");
assert.sameValue(resultDay.day, 24, "day is changed");
