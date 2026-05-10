// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.plaindate.from
description: Out-of-range months/days are rejected with overflow 'reject'
features: [Temporal]
---*/

assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 1,
  day: 32
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 2,
  day: 29
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 3,
  day: 32
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 4,
  day: 31
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 5,
  day: 32
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 6,
  day: 31
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 7,
  day: 32
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 8,
  day: 32
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 9,
  day: 31
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 10,
  day: 32
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 11,
  day: 31
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 12,
  day: 32
}, { overflow: 'reject' }));
assert.throws(RangeError, () => Temporal.PlainDate.from({
  year: 2021,
  month: 13,
  day: 5
}, { overflow: 'reject' }));
