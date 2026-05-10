// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tolocalestring
description: Omitting the locales argument defaults to the DateTimeFormat default
features: [Temporal]
---*/

const defaultFormatter = new Intl.DateTimeFormat([], Object.create(null));
const { calendar } = defaultFormatter.resolvedOptions();
const yearmonth = new Temporal.PlainYearMonth(2000, 5, calendar);
const expected = defaultFormatter.format(yearmonth);

const actualExplicit = yearmonth.toLocaleString(undefined);
assert.sameValue(actualExplicit, expected, "default locale is determined by Intl.DateTimeFormat");

const actualImplicit = yearmonth.toLocaleString();
assert.sameValue(actualImplicit, expected, "default locale is determined by Intl.DateTimeFormat");
