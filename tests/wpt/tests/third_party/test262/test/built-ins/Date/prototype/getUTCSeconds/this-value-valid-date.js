// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getutcseconds
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return SecFromTime(t).
---*/

var sec34 = 1467819394000;
var sec59 = 1467819419000;

assert.sameValue(new Date(sec34).getUTCSeconds(), 34, 'first millisecond');
assert.sameValue(
  new Date(sec34 - 1).getUTCSeconds(), 33, 'previous millisecond'
);
assert.sameValue(
  new Date(sec34 + 999).getUTCSeconds(), 34, 'final millisecond'
);
assert.sameValue(
  new Date(sec34 + 1000).getUTCSeconds(), 35, 'subsequent millisecond'
);

assert.sameValue(
  new Date(sec59).getUTCSeconds(), 59, 'first millisecond (minute boundary)'
);
assert.sameValue(
  new Date(sec59 - 1).getUTCSeconds(),
  58,
  'previous millisecond (minute boundary)'
);
assert.sameValue(
  new Date(sec59 + 99).getUTCSeconds(),
  59,
  'final millisecond (minute boundary)'
);
assert.sameValue(
  new Date(sec59 + 1000).getUTCSeconds(),
  0,
  'subsequent millisecond (minute boundary)'
);
