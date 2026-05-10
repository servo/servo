// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Adding months to maximum year should throw
features: [Temporal]
---*/

const maxYear = new Temporal.PlainDate(275760, 1, 1);
const duration = new Temporal.Duration(0, 5432, 5432, 0, 0, 0, 0, 0, 0, 0);
assert.throws(RangeError, () => maxYear.add(duration));

const minYear = new Temporal.PlainDate(-271821, 4, 19);
assert.throws(RangeError, () => minYear.add(duration.negated()));
