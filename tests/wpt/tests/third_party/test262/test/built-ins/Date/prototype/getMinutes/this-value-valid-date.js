// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getminutes
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return MinFromTime(LocalTime(t)).
---*/

assert.sameValue(new Date(2016, 6, 6, 14, 6).getMinutes(), 6, 'first millisecond');
assert.sameValue(
  new Date(2016, 6, 6, 14, 6, 0, -1).getMinutes(), 5, 'previous millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 6, 59, 999).getMinutes(), 6, 'final millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 6, 59, 1000).getMinutes(), 7, 'subsequent millisecond'
);

assert.sameValue(
  new Date(2016, 6, 6, 14, 59).getMinutes(), 59, 'first millisecond (hour boundary)'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 59, 0, -1).getMinutes(),
  58,
  'previous millisecond (hour boundary)'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 59, 59, 999).getMinutes(),
  59,
  'final millisecond (hour boundary)'
);
assert.sameValue(
  new Date(2016, 6, 6, 14, 59, 59, 1000).getMinutes(),
  0,
  'subsequent millisecond (hour boundary)'
);
