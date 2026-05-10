// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.utc
description: Conditional offset of provided `year` value
info: |
  1. Let y be ? ToNumber(year).
  [...]
  8. If y is not NaN and 0 ≤ ToInteger(y) ≤ 99, let yr be 1900+ToInteger(y);
     otherwise, let yr be y.
  9. Return TimeClip(MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli))). 
---*/

assert.sameValue(Date.UTC(-1, 0), -62198755200000, '-1 (no offset)');

assert.sameValue(Date.UTC(0, 0), -2208988800000, '+0');
assert.sameValue(Date.UTC(-0, 0), -2208988800000, '-0');
assert.sameValue(Date.UTC(-0.999999, 0), -2208988800000, '-0.999999');

assert.sameValue(Date.UTC(70, 0), 0, '70');
assert.sameValue(Date.UTC(70, 0), 0, '70.999999');

assert.sameValue(Date.UTC(99, 0), 915148800000, '99');
assert.sameValue(Date.UTC(99.999999, 0), 915148800000, '99.999999');

assert.sameValue(Date.UTC(100, 0), -59011459200000, '100 (no offset)');
