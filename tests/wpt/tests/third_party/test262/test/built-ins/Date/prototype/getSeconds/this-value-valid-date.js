// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getseconds
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return SecFromTime(LocalTime(t)).
---*/

assert.sameValue(
  new Date(2016, 6, 6, 14, 16, 30).getSeconds(),
  30,
  'first millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 16, 30, -1).getSeconds(), 29, 'previous millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 16, 30, 999).getSeconds(), 30, 'final millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 16, 30, 1000).getSeconds(),
  31,
  'subsequent millisecond'
);

assert.sameValue(
  new Date(2016, 6, 6, 14, 16, 59).getSeconds(),
  59,
  'first millisecond (minute boundary)'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 16, 59, -1).getSeconds(),
  58,
  'previous millisecond (minute boundary)'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 16, 59, 999).getSeconds(),
  59,
  'final millisecond (minute boundary)'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 16, 59, 1000).getSeconds(),
  0,
  'subsequent millisecond (minute boundary)'
);
