// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.gettimezoneoffset
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return (t - LocalTime(t)) / msPerMinute.
---*/

assert.sameValue(
  typeof new Date(0).getTimezoneOffset(), 'number', 'Unix epoch'
);

assert.sameValue(
  typeof new Date(8640000000000000).getTimezoneOffset(),
  'number',
  'latest representable time'
);

assert.sameValue(
  typeof new Date(-8640000000000000).getTimezoneOffset(),
  'number',
  'earliest representable time'
);

assert.sameValue(
  typeof new Date().getTimezoneOffset(), 'number', 'current time'
);
