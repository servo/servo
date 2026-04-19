// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getutcmonth
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return MonthFromTime(t).
---*/

var july31 = 1469923200000;
var dec31 = 1483142400000;
var dayMs = 24 * 60 * 60 * 1000;

assert.sameValue(new Date(july31).getUTCMonth(), 6, 'first millisecond');
assert.sameValue(
  new Date(july31 - 1).getUTCMonth(), 6, 'previous millisecond'
);
assert.sameValue(
  new Date(july31 + dayMs - 1).getUTCMonth(), 6, 'final millisecond'
);
assert.sameValue(
  new Date(july31 + dayMs).getUTCMonth(), 7, 'subsequent millisecond'
);

assert.sameValue(
  new Date(dec31).getUTCMonth(), 11, 'first millisecond (year boundary)'
);
assert.sameValue(
  new Date(dec31 - 1).getUTCMonth(), 11, 'previous millisecond (year boundary)'
);
assert.sameValue(
  new Date(dec31 + dayMs - 1).getUTCMonth(),
  11,
  'final millisecond (year boundary)'
);
assert.sameValue(
  new Date(dec31 + dayMs).getUTCMonth(),
  0,
  'subsequent millisecond (year boundary)'
);
