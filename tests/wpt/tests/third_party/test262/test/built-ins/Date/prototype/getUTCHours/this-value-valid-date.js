// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getutchours
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return HourFromTime(t).
---*/

var hour15 = 1467817200000;
var hour23 = 1467846000000;
var hourMs = 60 * 60 * 1000;

assert.sameValue(new Date(hour15).getUTCHours(), 15, 'first millisecond');
assert.sameValue(
  new Date(hour15 - 1).getUTCHours(), 14, 'previous millisecond'
);
assert.sameValue(
  new Date(hour15 + hourMs - 1).getUTCHours(), 15, 'final millisecond'
);
assert.sameValue(
  new Date(hour15 + hourMs).getUTCHours(), 16, 'subsequent millisecond'
);

assert.sameValue(
  new Date(hour23).getUTCHours(), 23, 'first millisecond (day boundary)'
);
assert.sameValue(
  new Date(hour23 - 1).getUTCHours(), 22, 'previous millisecond (day boundary)'
);
assert.sameValue(
  new Date(hour23 + hourMs - 1).getUTCHours(),
  23,
  'final millisecond (day boundary)'
);
assert.sameValue(
  new Date(hour23 + hourMs).getUTCHours(),
  0,
  'subsequent millisecond (day boundary)'
);
