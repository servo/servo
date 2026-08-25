// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getutcday
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return WeekDay(t).
---*/

var july6 = 1467763200000;
var july9 = 1468022400000;
var dayMs = 24 * 60 * 60 * 1000;

assert.sameValue(new Date(july6).getUTCDay(), 3, 'first millisecond');
assert.sameValue(
  new Date(july6 - 1).getUTCDay(), 2, 'previous millisecond'
);
assert.sameValue(
  new Date(july6 + dayMs - 1).getUTCDay(), 3, 'final millisecond'
);
assert.sameValue(
  new Date(july6 + dayMs).getUTCDay(), 4, 'subsequent millisecond'
);

assert.sameValue(
  new Date(july9).getUTCDay(), 6, 'first millisecond (week boundary)'
);
assert.sameValue(
  new Date(july9 - 1).getUTCDay(), 5, 'previous millisecond (week boundary)'
);
assert.sameValue(
  new Date(july9 + dayMs - 1).getUTCDay(),
  6,
  'final millisecond (week boundary)'
);
assert.sameValue(
  new Date(july9 + dayMs).getUTCDay(),
  0,
  'subsequent millisecond (week boundary)'
);
