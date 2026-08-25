// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: equals() takes the calendar into account
features: [Temporal]
---*/

const ym1 = new Temporal.PlainYearMonth(2000, 1, "iso8601", 1);
const ym2 = new Temporal.PlainYearMonth(2000, 1, "iso8601", 1);
assert(ym1.equals(ym2), "Equal if calendars and ISO dates are equal");

const ym3 = new Temporal.PlainYearMonth(2000, 1, "iso8601", 1);
const ym4 = new Temporal.PlainYearMonth(2000, 1, "gregory", 1);
assert(!ym3.equals(ym4), "Unequal if ISO dates are equal but calendars are not");
