// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  If `end` is undefined, set final position to `this.length`.
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  11. If end is undefined, let relativeEnd be len; else let relativeEnd be
  ToInteger(end).
  ...
includes: [compareArray.js]
---*/

assert.compareArray(
  [0, 1, 2, 3].copyWithin(0, 1, undefined), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(0, 1, undefined) must return [1, 2, 3, 3]'
);

assert.compareArray(
  [0, 1, 2, 3].copyWithin(0, 1), [1, 2, 3, 3],
  '[0, 1, 2, 3].copyWithin(0, 1) must return [1, 2, 3, 3]'
);
