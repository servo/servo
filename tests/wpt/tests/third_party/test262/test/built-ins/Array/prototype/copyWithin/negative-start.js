// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Set values with negative start argument.
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  10. If relativeStart < 0, let from be max((len + relativeStart),0); else let
  from be min(relativeStart, len).
  ...
includes: [compareArray.js]
---*/

assert.compareArray(
  [0, 1, 2, 3].copyWithin(0, -1), [3, 1, 2, 3],
  '[0, 1, 2, 3].copyWithin(0, -1) must return [3, 1, 2, 3]'
);

assert.compareArray(
  [0, 1, 2, 3, 4].copyWithin(2, -2), [0, 1, 3, 4, 4],
  '[0, 1, 2, 3, 4].copyWithin(2, -2) must return [0, 1, 3, 4, 4]'
);

assert.compareArray(
  [0, 1, 2, 3, 4].copyWithin(1, -2), [0, 3, 4, 3, 4],
  '[0, 1, 2, 3, 4].copyWithin(1, -2) must return [0, 3, 4, 3, 4]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(-1, -2), [0, 1, 2, 2],
  '[0, 1, 2, 3].copyWithin(-1, -2) must return [0, 1, 2, 2]'
);

assert.compareArray(
  [0, 1, 2, 3, 4].copyWithin(-2, -3), [0, 1, 2, 2, 3],
  '[0, 1, 2, 3, 4].copyWithin(-2, -3) must return [0, 1, 2, 2, 3]'
);

assert.compareArray(
  [0, 1, 2, 3, 4].copyWithin(-5, -2), [3, 4, 2, 3, 4],
  '[0, 1, 2, 3, 4].copyWithin(-5, -2) must return [3, 4, 2, 3, 4]'
);
