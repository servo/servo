// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getday
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return WeekDay(LocalTime(t)).
---*/

assert.sameValue(new Date(2016, 6, 6).getDay(), 3, 'first millisecond');
assert.sameValue(
  new Date(2016, 6, 6, 0, 0, 0, -1).getDay(), 2, 'previous millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 23, 59, 59, 999).getDay(), 3, 'final millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 23, 59, 59, 1000).getDay(), 4, 'subsequent millisecond'
);

assert.sameValue(
  new Date(2016, 6, 9).getDay(), 6, 'first millisecond (week boundary)'
);
assert.sameValue(
  new Date(2016, 6, 9, 0, 0, 0, -1).getDay(),
  5,
  'previous millisecond (week boundary)'
);
assert.sameValue(
  new Date(2016, 6, 9, 23, 59, 59, 999).getDay(),
  6,
  'final millisecond (week boundary)'
);
assert.sameValue(
  new Date(2016, 6, 9, 23, 59, 59, 1000).getDay(),
  0,
  'subsequent millisecond (week boundary)'
);
