// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getyear
es6id: B.2.4.1
es5id: B.2.4
description: >
    Return value for objects with numeric value in [[DateValue]] internal slot
info: |
    1. Let t be ? thisTimeValue(this value).
    2. If t is NaN, return NaN.
    3. Return YearFromTime(LocalTime(t)) - 1900.
---*/

assert.sameValue(new Date(1899, 0).getYear(), -1, '1899: first millisecond');
assert.sameValue(
  new Date(1899, 11, 31, 23, 59, 59, 999).getYear(),
  -1,
  '1899: final millisecond'
);

assert.sameValue(new Date(1900, 0).getYear(), 0, '1900: first millisecond');
assert.sameValue(
  new Date(1900, 11, 31, 23, 59, 59, 999).getYear(),
  0,
  '1900: final millisecond'
);

assert.sameValue(new Date(1970, 0).getYear(), 70, '1970: first millisecond');
assert.sameValue(
  new Date(1970, 11, 31, 23, 59, 59, 999).getYear(),
  70,
  '1970: final millisecond'
);

assert.sameValue(new Date(2000, 0).getYear(), 100, '2000: first millisecond');
assert.sameValue(
  new Date(2000, 11, 31, 23, 59, 59, 999).getYear(),
  100,
  '2000: final millisecond'
);
