// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tolocalestring
description: dateStyle present but undefined
features: [Temporal]
---*/

const defaultFormatter = new Intl.DateTimeFormat("en");
const { calendar } = defaultFormatter.resolvedOptions();
const yearmonth = new Temporal.PlainYearMonth(2000, 5, calendar);
const expected = defaultFormatter.format(yearmonth);

const actual = yearmonth.toLocaleString("en", { dateStyle: undefined });
assert.sameValue(actual, expected, "dateStyle undefined is the same as being absent");
