// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getmonth
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return MonthFromTime(LocalTime(t)).
---*/

assert.sameValue(new Date(2016, 6).getMonth(), 6, 'first millisecond');
assert.sameValue(
  new Date(2016, 6, 0, 0, 0, 0, -1).getMonth(), 5, 'previous millisecond'
);
assert.sameValue(
  new Date(2016, 6, 31, 23, 59, 59, 999).getMonth(), 6, 'final millisecond'
);
assert.sameValue(
  new Date(2016, 6, 31, 23, 59, 59, 1000).getMonth(), 7, 'subsequent millisecond'
);

assert.sameValue(
  new Date(2016, 11, 31).getMonth(), 11, 'first millisecond (year boundary)'
);
assert.sameValue(
  new Date(2016, 11, 0, 0, 0, 0, -1).getMonth(),
  10,
  'previous millisecond (year boundary)'
);
assert.sameValue(
  new Date(2016, 11, 31, 23, 59, 59, 999).getMonth(),
  11,
  'final millisecond (year boundary)'
);
assert.sameValue(
  new Date(2016, 11, 31, 23, 59, 59, 1000).getMonth(),
  0,
  'subsequent millisecond (year boundary)'
);
