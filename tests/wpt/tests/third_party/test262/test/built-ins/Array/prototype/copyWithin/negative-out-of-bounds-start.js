// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Set values with out of bounds negative start argument.
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  10. If relativeStart < 0, let from be max((len + relativeStart),0); else let
  from be min(relativeStart, len).
  ...
includes: [compareArray.js]
---*/

assert.compareArray(
  [0, 1, 2, 3].copyWithin(0, -10), [0, 1, 2, 3],
  '[0, 1, 2, 3].copyWithin(0, -10) must return [0, 1, 2, 3]'
);

assert.compareArray(
  [1, 2, 3, 4, 5].copyWithin(0, -Infinity), [1, 2, 3, 4, 5],
  '[1, 2, 3, 4, 5].copyWithin(0, -Infinity) must return [1, 2, 3, 4, 5]'
);

assert.compareArray(
  [0, 1, 2, 3, 4].copyWithin(2, -10), [0, 1, 0, 1, 2],
  '[0, 1, 2, 3, 4].copyWithin(2, -10) must return [0, 1, 0, 1, 2]'
);

assert.compareArray(
  [1, 2, 3, 4, 5].copyWithin(2, -Infinity), [1, 2, 1, 2, 3],
  '[1, 2, 3, 4, 5].copyWithin(2, -Infinity) must return [1, 2, 1, 2, 3]'
);


assert.compareArray(
  [0, 1, 2, 3, 4].copyWithin(10, -10), [0, 1, 2, 3, 4],
  '[0, 1, 2, 3, 4].copyWithin(10, -10) must return [0, 1, 2, 3, 4]'
);

assert.compareArray(
  [1, 2, 3, 4, 5].copyWithin(10, -Infinity), [1, 2, 3, 4, 5],
  '[1, 2, 3, 4, 5].copyWithin(10, -Infinity) must return [1, 2, 3, 4, 5]'
);


assert.compareArray(
  [0, 1, 2, 3].copyWithin(-9, -10), [0, 1, 2, 3],
  '[0, 1, 2, 3].copyWithin(-9, -10) must return [0, 1, 2, 3]'
);

assert.compareArray(
  [1, 2, 3, 4, 5].copyWithin(-9, -Infinity), [1, 2, 3, 4, 5],
  '[1, 2, 3, 4, 5].copyWithin(-9, -Infinity) must return [1, 2, 3, 4, 5]'
);
