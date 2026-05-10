// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate
description: Limits for the PlainDate constructor.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Year limits

assert.throws(RangeError, () => new Temporal.PlainDate(-271821, 4, 18), "min");
assert.throws(RangeError, () => new Temporal.PlainDate(275760, 9, 14), "max");
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(-271821, 4, 19),
  -271821, 4, "M04", 19, "min");
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(275760, 9, 13),
  275760, 9, "M09", 13, "max");

// Monthday limits

TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 1, 31), 2021, 1, 'M01', 31);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 2, 28), 2021, 2, 'M02', 28);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 3, 31), 2021, 3, 'M03', 31);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 4, 30), 2021, 4, 'M04', 30);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 5, 31), 2021, 5, 'M05', 31);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 6, 30), 2021, 6, 'M06', 30);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 7, 31), 2021, 7, 'M07', 31);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 8, 31), 2021, 8, 'M08', 31);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 9, 30), 2021, 9, 'M09', 30);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 10, 31), 2021, 10, 'M10', 31);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 11, 30), 2021, 11, 'M11', 30);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2021, 12, 31), 2021, 12, 'M12', 31);
TemporalHelpers.assertPlainDate(new Temporal.PlainDate(2004, 2, 29), 2004, 2, 'M02', 29);
assert.throws(RangeError, () => new Temporal.PlainDate(1900, 2, 29));
assert.throws(RangeError, () => new Temporal.PlainDate(2001, 2, 29));
assert.throws(RangeError, () => new Temporal.PlainDate(2002, 2, 29));
assert.throws(RangeError, () => new Temporal.PlainDate(2003, 2, 29));
assert.throws(RangeError, () => new Temporal.PlainDate(2100, 2, 29));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 1, 32));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 2, 29));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 3, 32));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 4, 31));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 5, 32));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 6, 31));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 7, 32));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 8, 32));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 9, 31));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 10, 32));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 11, 31));
assert.throws(RangeError, () => new Temporal.PlainDate(2021, 12, 32));
