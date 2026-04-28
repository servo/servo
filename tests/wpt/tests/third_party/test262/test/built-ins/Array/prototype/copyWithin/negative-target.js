// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Set values with negative target argument.
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  7. If relativeTarget < 0, let to be max((len + relativeTarget),0); else let to
  be min(relativeTarget, len).
  ...
includes: [compareArray.js]
---*/

assert.compareArray(
  [0, 1, 2, 3].copyWithin(-1, 0), [0, 1, 2, 0],
  '[0, 1, 2, 3].copyWithin(-1, 0) must return [0, 1, 2, 0]'
);

assert.compareArray(
  [0, 1, 2, 3, 4].copyWithin(-2, 2), [0, 1, 2, 2, 3],
  '[0, 1, 2, 3, 4].copyWithin(-2, 2) must return [0, 1, 2, 2, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(-1, 2), [0, 1, 2, 2],
  '[0, 1, 2, 3].copyWithin(-1, 2) must return [0, 1, 2, 2]'
);
