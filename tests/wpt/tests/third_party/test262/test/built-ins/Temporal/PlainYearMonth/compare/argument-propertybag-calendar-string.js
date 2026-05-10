// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: A calendar ID is valid input for Calendar
features: [Temporal]
---*/

const calendar = "iso8601";

const arg = { year: 2019, monthCode: "M06", calendar };

const result1 = Temporal.PlainYearMonth.compare(arg, new Temporal.PlainYearMonth(2019, 6));
assert.sameValue(result1, 0, `Calendar created from string "${arg}" (first argument)`);

const result2 = Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2019, 6), arg);
assert.sameValue(result2, 0, `Calendar created from string "${arg}" (second argument)`);
