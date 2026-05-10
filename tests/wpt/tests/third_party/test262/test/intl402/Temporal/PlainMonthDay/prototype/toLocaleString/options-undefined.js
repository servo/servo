// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tolocalestring
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const defaultFormatter = new Intl.DateTimeFormat('en', Object.create(null));
const { calendar } = defaultFormatter.resolvedOptions();
const monthday = new Temporal.PlainMonthDay(5, 2, calendar);
const expected = defaultFormatter.format(monthday);

const actualExplicit = monthday.toLocaleString('en', undefined);
assert.sameValue(actualExplicit, expected, "default locale is determined by Intl.DateTimeFormat");

const actualImplicit = monthday.toLocaleString('en');
assert.sameValue(actualImplicit, expected, "default locale is determined by Intl.DateTimeFormat");
