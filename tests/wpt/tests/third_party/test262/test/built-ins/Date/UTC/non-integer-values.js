// Copyright (C) 2018 Viktor Mukhachev. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.utc
description: non-integer values are converted to integers using `ToInteger`
info: |
  [...]
  Return TimeClip(MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli))).

  #sec-timeclip

  Let clippedTime be ! ToInteger(time).

  #sec-makeday

  Let y be ! ToInteger(year).
  Let m be ! ToInteger(month).
  Let dt be ! ToInteger(date).

  #sec-maketime

  Let h be ! ToInteger(hour).
  Let m be ! ToInteger(min).
  Let s be ! ToInteger(sec).
  Let milli be ! ToInteger(ms).
---*/

assert.sameValue(Date.UTC(1970.9, 0.9, 1.9, 0.9, 0.9, 0.9, 0.9), 0, 'positive non-integer values');
assert.sameValue(Date.UTC(-1970.9, -0.9, -0.9, -0.9, -0.9, -0.9, -0.9), -124334438400000, 'negative non-integer values');
