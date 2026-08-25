// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getutcdate
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return DateFromTime(t).
---*/

var july6 = 1467763200000;
var feb29 = 1456704000000;
var dayMs = 24 * 60 * 60 * 1000;

assert.sameValue(new Date(july6).getUTCDate(), 6, 'first millisecond');
assert.sameValue(
  new Date(july6 - 1).getUTCDate(), 5, 'previous millisecond'
);
assert.sameValue(
  new Date(july6 + dayMs - 1).getUTCDate(), 6, 'final millisecond'
);
assert.sameValue(
  new Date(july6 + dayMs).getUTCDate(), 7, 'subsequent millisecond'
);

assert.sameValue(
  new Date(feb29).getUTCDate(), 29, 'first millisecond (month boundary)'
);
assert.sameValue(
  new Date(feb29 - 1).getUTCDate(), 28, 'previous millisecond (month boundary)'
);
assert.sameValue(
  new Date(feb29 + dayMs - 1).getUTCDate(),
  29,
  'final millisecond (month boundary)'
);
assert.sameValue(
  new Date(feb29 + dayMs).getUTCDate(),
  1,
  'subsequent millisecond (month boundary)'
);
