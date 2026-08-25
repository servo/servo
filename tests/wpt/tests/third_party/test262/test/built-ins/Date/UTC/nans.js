// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.utc
description: NaN value handling
info: |
  1. Let y be ? ToNumber(year).
  2. Let m be ? ToNumber(month).
  3. If date is supplied, let dt be ? ToNumber(date); else let dt be 1.
  4. If hours is supplied, let h be ? ToNumber(hours); else let h be 0.
  5. If minutes is supplied, let min be ? ToNumber(minutes); else let min be 0.
  6. If seconds is supplied, let s be ? ToNumber(seconds); else let s be 0.
  7. If ms is supplied, let milli be ? ToNumber(ms); else let milli be 0.
  8. If y is not NaN and 0 ≤ ToInteger(y) ≤ 99, let yr be 1900+ToInteger(y);
     otherwise, let yr be y.
  9. Return TimeClip(MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli))).
---*/

assert.sameValue(Date.UTC(NaN), NaN, 'year');
assert.sameValue(Date.UTC(NaN, 0), NaN, 'year');
assert.sameValue(Date.UTC(1970, NaN), NaN, 'month');
assert.sameValue(Date.UTC(1970, 0, NaN), NaN, 'date');
assert.sameValue(Date.UTC(1970, 0, 1, NaN), NaN, 'hours');
assert.sameValue(Date.UTC(1970, 0, 1, 0, NaN), NaN, 'minutes');
assert.sameValue(Date.UTC(1970, 0, 1, 0, 0, NaN), NaN, 'seconds');
assert.sameValue(Date.UTC(1970, 0, 1, 0, 0, 0, NaN), NaN, 'ms');
