// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Calls to PYM.until cast arguments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const nov94 = new Temporal.PlainYearMonth(1994, 11);
const jun13 = new Temporal.PlainYearMonth(2013, 6);
const diff = nov94.until(jun13);

TemporalHelpers.assertDurationsEqual(nov94.until({ year: 2013, month: 6 }), diff, "Casts object argument");
TemporalHelpers.assertDurationsEqual(nov94.until("2013-06"), diff, "Casts string argument");
