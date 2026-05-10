// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Subtracting months from minimum year should throw
features: [Temporal]
---*/

const minYear = new Temporal.PlainDate(-271821, 4, 19);
const duration = new Temporal.Duration(0, 5432, 5432, 0, 0, 0, 0, 0, 0, 0);
assert.throws(RangeError, () => minYear.subtract(duration));

const maxYear = new Temporal.PlainDate(275760, 1, 1);
assert.throws(RangeError, () => maxYear.subtract(duration.negated()));
