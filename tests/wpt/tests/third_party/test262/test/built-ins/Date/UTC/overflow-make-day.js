// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.utc
description: Values specified to MakeDay exceed their calendar boundaries
info: |
  [...]
  9. Return TimeClip(MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli))). 

  MakeDay (year, month, date)

  [...]
  5. Let ym be y + floor(m / 12).
  [...]
  7. Find a value t such that YearFromTime(t) is ym and MonthFromTime(t) is mn
     and DateFromTime(t) is 1; but if this is not possible (because some
     argument is out of range), return NaN.
  8. Return Day(t) + dt - 1.
---*/

assert.sameValue(Date.UTC(2016, 12), 1483228800000, 'month: 12');
assert.sameValue(Date.UTC(2016, 13), 1485907200000, 'month: 13');
assert.sameValue(Date.UTC(2016, 144), 1830297600000, 'month: 144');

assert.sameValue(Date.UTC(2016, 0, 33), 1454371200000, 'day greater than month');
assert.sameValue(Date.UTC(2016, 2, -27), 1454371200000, 'day negative value');
