// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getutcfullyear
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return YearFromTime(t).
---*/

var dec31 = 1483142400000;
var dayMs = 24 * 60 * 60 * 1000;

assert.sameValue(new Date(dec31).getUTCFullYear(), 2016, 'first millisecond');
assert.sameValue(
  new Date(dec31 - 1).getUTCFullYear(), 2016, 'previous millisecond'
);
assert.sameValue(
  new Date(dec31 + dayMs - 1).getUTCFullYear(), 2016, 'final millisecond'
);
assert.sameValue(
  new Date(dec31 + dayMs).getUTCFullYear(), 2017, 'subsequent millisecond'
);
