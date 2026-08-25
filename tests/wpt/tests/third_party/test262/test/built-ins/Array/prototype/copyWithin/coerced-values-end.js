// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  end argument is coerced to an integer values.
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  11. If end is undefined, let relativeEnd be len; else let relativeEnd be
  ToInteger(end).
  ...
includes: [compareArray.js]
---*/

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, 0, null), [0, 1, 2, 3],
  '[0, 1, 2, 3].copyWithin(1, 0, null) must return [0, 1, 2, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, 0, NaN), [0, 1, 2, 3],
  '[0, 1, 2, 3].copyWithin(1, 0, NaN) must return [0, 1, 2, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, 0, false), [0, 1, 2, 3],
  '[0, 1, 2, 3].copyWithin(1, 0, false) must return [0, 1, 2, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, 0, true), [0, 0, 2, 3],
  '[0, 1, 2, 3].copyWithin(1, 0, true) must return [0, 0, 2, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, 0, '-2'), [0, 0, 1, 3],
  '[0, 1, 2, 3].copyWithin(1, 0, "-2") must return [0, 0, 1, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(1, 0, -2.5), [0, 0, 1, 3],
  '[0, 1, 2, 3].copyWithin(1, 0, -2.5) must return [0, 0, 1, 3]'
);
