// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getmilliseconds
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return msFromTime(LocalTime(t)).
---*/

assert.sameValue(
  new Date(2016, 6, 6).getMilliseconds(), 0, 'first millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 0, 0, 0, -1).getMilliseconds(),
  999,
  'previous millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 23, 59, 59, 999).getMilliseconds(),
  999,
  'final millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 23, 59, 59, 1000).getMilliseconds(),
  0,
  'subsequent millisecond'
);
