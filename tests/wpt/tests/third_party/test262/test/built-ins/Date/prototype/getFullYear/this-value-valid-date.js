// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getfullyear
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return YearFromTime(LocalTime(t)).
---*/

assert.sameValue(new Date(2016, 0).getFullYear(), 2016, 'first millisecond');
assert.sameValue(
  new Date(2016, 0, 1, 0, 0, 0, -1).getFullYear(), 2015, 'previous millisecond'
);
assert.sameValue(
  new Date(2016, 11, 31, 23, 59, 59, 999).getFullYear(),
  2016,
  'final millisecond'
);
assert.sameValue(
  new Date(2016, 11, 31, 23, 59, 59, 1000).getFullYear(),
  2017,
  'subsequent millisecond'
);
