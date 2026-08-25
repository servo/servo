// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getdate
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return DateFromTime(LocalTime(t)).
---*/

assert.sameValue(new Date(2016, 6, 6).getDate(), 6, 'first millisecond');
assert.sameValue(
  new Date(2016, 6, 6, 0, 0, 0, -1).getDate(), 5, 'previous millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 23, 59, 59, 999).getDate(), 6, 'final millisecond'
);
assert.sameValue(
  new Date(2016, 6, 6, 23, 59, 59, 1000).getDate(), 7, 'subsequent millisecond'
);

assert.sameValue(
  new Date(2016, 1, 29).getDate(), 29, 'first millisecond (month boundary)'
);
assert.sameValue(
  new Date(2016, 1, 29, 0, 0, 0, -1).getDate(),
  28,
  'previous millisecond (month boundary)'
);
assert.sameValue(
  new Date(2016, 1, 29, 23, 59, 59, 999).getDate(),
  29,
  'final millisecond (month boundary)'
);
assert.sameValue(
  new Date(2016, 1, 29, 23, 59, 59, 1000).getDate(),
  1,
  'subsequent millisecond (month boundary)'
);
