// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.utc
description: Return value of `Date.UTC`
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

assert.sameValue(Date.UTC(1970), 0, '1970');

assert.sameValue(Date.UTC(1970, 0), 0, '1970, 0');
assert.sameValue(Date.UTC(2016, 0), 1451606400000, '2016, 0');
assert.sameValue(Date.UTC(2016, 6), 1467331200000, '2016, 6');

assert.sameValue(Date.UTC(2016, 6, 1), 1467331200000, '2016, 6, 1');
assert.sameValue(Date.UTC(2016, 6, 5), 1467676800000, '2016, 6, 5');

assert.sameValue(Date.UTC(2016, 6, 5, 0), 1467676800000, '2016, 6, 5, 0');
assert.sameValue(Date.UTC(2016, 6, 5, 15), 1467730800000, '2016, 6, 5, 15');

assert.sameValue(
  Date.UTC(2016, 6, 5, 15, 0), 1467730800000, '2016, 6, 5, 15, 0'
);
assert.sameValue(
  Date.UTC(2016, 6, 5, 15, 34), 1467732840000, '2016, 6, 5, 15, 34'
);

assert.sameValue(
  Date.UTC(2016, 6, 5, 15, 34, 0), 1467732840000, '2016, 6, 5, 15, 34, 0'
);
assert.sameValue(
  Date.UTC(2016, 6, 5, 15, 34, 45), 1467732885000, '2016, 6, 5, 15, 34, 45'
);


assert.sameValue(
  Date.UTC(2016, 6, 5, 15, 34, 45, 0),
  1467732885000,
  '2016, 6, 5, 15, 34, 45, 0'
);
assert.sameValue(
  Date.UTC(2016, 6, 5, 15, 34, 45, 876),
  1467732885876,
  '2016, 6, 5, 15, 34, 45, 0'
);
