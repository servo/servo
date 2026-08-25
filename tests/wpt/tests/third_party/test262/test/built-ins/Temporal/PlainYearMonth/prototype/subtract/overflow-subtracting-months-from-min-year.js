// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Subtracting months from minimum year should throw
features: [Temporal]
---*/

const minYear = new Temporal.PlainYearMonth(-271821, 4);
const duration = new Temporal.Duration(0, 5432, 5432, 0, 0, 0, 0, 0, 0, 0);
assert.throws(RangeError, () => minYear.subtract(duration));

const maxYear = new Temporal.PlainYearMonth(275760, 1);
assert.throws(RangeError, () => maxYear.subtract(duration.negated()));
