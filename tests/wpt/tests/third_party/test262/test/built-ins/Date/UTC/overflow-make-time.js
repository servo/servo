// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.utc
description: Values specified to MakeTime exceed their time boundaries
info: |
  [...]
  9. Return TimeClip(MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli))). 

  MakeTime (hour, min, sec, ms)

  1. If hour is not finite or min is not finite or sec is not finite or ms is
     not finite, return NaN.
  2. Let h be ToInteger(hour).
  3. Let m be ToInteger(min).
  4. Let s be ToInteger(sec).
  5. Let milli be ToInteger(ms).
  6. Let t be h * msPerHour + m * msPerMinute + s * msPerSecond + milli,
     performing the arithmetic according to IEEE 754-2008 rules (that is, as if
     using the ECMAScript operators * and +).
  7. Return t.
---*/

assert.sameValue(Date.UTC(2016, 6, 5, -1), 1467673200000, 'hour: -1');
assert.sameValue(Date.UTC(2016, 6, 5, 24), 1467763200000, 'hour: 24');
assert.sameValue(Date.UTC(2016, 6, 5, 0, -1), 1467676740000, 'minute: -1');
assert.sameValue(Date.UTC(2016, 6, 5, 0, 60), 1467680400000, 'minute: 60');
assert.sameValue(Date.UTC(2016, 6, 5, 0, 0, -1), 1467676799000, 'second: -1');
assert.sameValue(Date.UTC(2016, 6, 5, 0, 0, 60), 1467676860000, 'second: 60');
assert.sameValue(
  Date.UTC(2016, 6, 5, 0, 0, 0, -1), 1467676799999, 'millisecond: -1'
);
assert.sameValue(
  Date.UTC(2016, 6, 5, 0, 0, 0, 1000), 1467676801000, 'millisecond: 1000'
);
