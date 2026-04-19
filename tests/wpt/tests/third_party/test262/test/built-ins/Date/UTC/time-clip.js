// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.utc
description: Time clipping
info: |
  [...]
  9. Return TimeClip(MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli))). 

  TimeClip (time)

  1. If time is not finite, return NaN.
  2. If abs(time) > 8.64 × 1015, return NaN.
---*/

assert.notSameValue(Date.UTC(275760, 8, 13, 0, 0, 0, 0), NaN);
assert.sameValue(Date.UTC(275760, 8, 13, 0, 0, 0, 1), NaN);
