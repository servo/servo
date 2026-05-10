// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Calls to PYM.since cast arguments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const nov94 = new Temporal.PlainYearMonth(1994, 11);
const jun13 = new Temporal.PlainYearMonth(2013, 6);
const diff = jun13.since(nov94);

TemporalHelpers.assertDurationsEqual(jun13.since({ year: 1994, month: 11 }), diff, 'Casts object argument');
TemporalHelpers.assertDurationsEqual(jun13.since('1994-11'), diff, 'Casts string argument');
