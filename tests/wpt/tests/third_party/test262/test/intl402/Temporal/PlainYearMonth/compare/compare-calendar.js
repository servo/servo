// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: compare() does not take the calendar into account
features: [Temporal]
---*/

const ym1 = new Temporal.PlainYearMonth(2000, 1, "iso8601", 1);
const ym2 = new Temporal.PlainYearMonth(2000, 1, "gregory", 1);
assert.sameValue(Temporal.PlainYearMonth.compare(ym1, ym2), 0);
