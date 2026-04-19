// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Subtracting months from minimum year should throw
features: [Temporal]
---*/

const minYear = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1);
const duration = new Temporal.Duration(0, 5432, 5432, 0, 0, 0, 0, 0, 0, 0);
assert.throws(RangeError, () => minYear.subtract(duration));

const maxYear = new Temporal.PlainDateTime(275760, 1, 1);
assert.throws(RangeError, () => maxYear.subtract(duration.negated()));
