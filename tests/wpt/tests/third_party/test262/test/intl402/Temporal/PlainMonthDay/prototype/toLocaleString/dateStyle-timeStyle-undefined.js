// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tolocalestring
description: dateStyle present but undefined
features: [Temporal]
---*/

const defaultFormatter = new Intl.DateTimeFormat("en");
const { calendar } = defaultFormatter.resolvedOptions();
const monthday = new Temporal.PlainMonthDay(5, 2, calendar);
const expected = defaultFormatter.format(monthday);

const actual = monthday.toLocaleString("en", { dateStyle: undefined });
assert.sameValue(actual, expected, "dateStyle undefined is the same as being absent");
