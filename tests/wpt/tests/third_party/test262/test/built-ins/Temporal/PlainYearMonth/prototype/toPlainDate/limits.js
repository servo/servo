// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.toplaindate
description: Throws a RangeError if the resulting PlainDate is out of range
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const min = Temporal.PlainYearMonth.from("-271821-04");
assert.throws(RangeError, () => min.toPlainDate({ day: 18 }), "min");
TemporalHelpers.assertPlainDate(min.toPlainDate({ day: 19 }),
  -271821, 4, "M04", 19, "min");

const max = Temporal.PlainYearMonth.from("+275760-09");
assert.throws(RangeError, () => max.toPlainDate({ day: 14 }), "max");
TemporalHelpers.assertPlainDate(max.toPlainDate({ day: 13 }),
  275760, 9, "M09", 13, "max");
