// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Since observes symmetry with until
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const nov94 = new Temporal.PlainYearMonth(1994, 11);
const jun13 = new Temporal.PlainYearMonth(2013, 6);
const diff = jun13.since(nov94);

TemporalHelpers.assertDurationsEqual(diff, nov94.until(jun13), 'Since is inverse of until');
