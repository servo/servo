// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.gethours
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return HourFromTime(LocalTime(t)).
---*/

assert.sameValue(new Date(2016, 6, 6, 13).getHours(), 13, 'first millisecond');
assert.sameValue(
  new Date(2016, 6, 6, 13, 0, 0, -1).getHours(), 12, 'previous millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 13, 59, 59, 999).getHours(),
  13,
  'final millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 13, 59, 59, 1000).getHours(),
  14,
  'subsequent millisecond'
);

assert.sameValue(
  new Date(2016, 6, 6, 23).getHours(), 23, 'first millisecond (hour boundary)'
);
assert.sameValue(
  new Date(2016, 6, 6, 23, 0, 0, -1).getHours(),
  22,
  'previous millisecond (hour boundary)'
);
assert.sameValue(
  new Date(2016, 6, 6, 23, 59, 59, 999).getHours(),
  23,
  'final millisecond (hour boundary)'
);
assert.sameValue(
  new Date(2016, 6, 6, 23, 59, 59, 1000).getHours(),
  0,
  'subsequent millisecond (hour boundary)'
);
