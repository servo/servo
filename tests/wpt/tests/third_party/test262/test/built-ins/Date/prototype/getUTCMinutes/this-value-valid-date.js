// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.getutcminutes
description: Return value for valid dates
info: |
  1. Let t be ? thisTimeValue(this value).
  2. If t is NaN, return NaN.
  3. Return MinFromTime(t).
---*/

var threeTwentyTwo = 1467818520000;
var threeFiftyNine = 1467820740000;
var minMs = 60 * 1000;

assert.sameValue(
  new Date(threeTwentyTwo).getUTCMinutes(), 22, 'first millisecond'
);
assert.sameValue(
  new Date(threeTwentyTwo - 1).getUTCMinutes(), 21, 'previous millisecond'
);
assert.sameValue(
  new Date(threeTwentyTwo + minMs - 1).getUTCMinutes(), 22, 'final millisecond'
);
assert.sameValue(
  new Date(threeTwentyTwo + minMs).getUTCMinutes(),
  23,
  'subsequent millisecond'
);

assert.sameValue(
  new Date(threeFiftyNine).getUTCMinutes(),
  59,
  'first millisecond (day boundary)'
);
assert.sameValue(
  new Date(threeFiftyNine - 1).getUTCMinutes(),
  58,
  'previous millisecond (day boundary)'
);
assert.sameValue(
  new Date(threeFiftyNine + minMs - 1).getUTCMinutes(),
  59,
  'final millisecond (day boundary)'
);
assert.sameValue(
  new Date(threeFiftyNine + minMs).getUTCMinutes(),
  0,
  'subsequent millisecond (day boundary)'
);
