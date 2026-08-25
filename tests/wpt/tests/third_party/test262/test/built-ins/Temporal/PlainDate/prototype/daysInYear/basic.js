// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindate.prototype.daysinyear
description: Basic tests for daysInYear.
features: [Temporal]
---*/

assert.sameValue((new Temporal.PlainDate(1976, 11, 18)).daysInYear, 366, "leap year");
assert.sameValue((new Temporal.PlainDate(1977, 11, 18)).daysInYear, 365, "non-leap year");
assert.sameValue((new Temporal.PlainDate(1995, 7, 15)).daysInYear, 365);
assert.sameValue((new Temporal.PlainDate(1996, 7, 15)).daysInYear, 366);
assert.sameValue((new Temporal.PlainDate(1997, 7, 15)).daysInYear, 365);
assert.sameValue((new Temporal.PlainDate(1998, 7, 15)).daysInYear, 365);
assert.sameValue((new Temporal.PlainDate(1999, 7, 15)).daysInYear, 365);
assert.sameValue((new Temporal.PlainDate(2000, 7, 15)).daysInYear, 366);
assert.sameValue((new Temporal.PlainDate(2001, 7, 15)).daysInYear, 365);
assert.sameValue((new Temporal.PlainDate(2002, 7, 15)).daysInYear, 365);
assert.sameValue((new Temporal.PlainDate(2003, 7, 15)).daysInYear, 365);
assert.sameValue((new Temporal.PlainDate(2004, 7, 15)).daysInYear, 366);
assert.sameValue((new Temporal.PlainDate(2005, 7, 15)).daysInYear, 365);
assert.sameValue(Temporal.PlainDate.from('2019-03-18').daysInYear, 365);
assert.sameValue(Temporal.PlainDate.from('2020-03-18').daysInYear, 366);
assert.sameValue(Temporal.PlainDate.from('2021-03-18').daysInYear, 365);
assert.sameValue(Temporal.PlainDate.from('2022-03-18').daysInYear, 365);
assert.sameValue(Temporal.PlainDate.from('2023-03-18').daysInYear, 365);
assert.sameValue(Temporal.PlainDate.from('2024-03-18').daysInYear, 366);
assert.sameValue(Temporal.PlainDate.from('2025-03-18').daysInYear, 365);
assert.sameValue(Temporal.PlainDate.from('2026-03-18').daysInYear, 365);
