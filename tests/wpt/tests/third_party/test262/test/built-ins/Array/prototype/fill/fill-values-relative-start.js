// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.fill
description: >
  Fills all the elements from a with a custom start index.
info: |
  22.1.3.6 Array.prototype.fill (value [ , start [ , end ] ] )

  ...
  7. If relativeStart < 0, let k be max((len + relativeStart),0); else let k be
  min(relativeStart, len).
  ...
includes: [compareArray.js]
---*/

assert.compareArray([0, 0, 0].fill(8, 1), [0, 8, 8],
  '[0, 0, 0].fill(8, 1) must return [0, 8, 8]'
);

assert.compareArray([0, 0, 0].fill(8, 4), [0, 0, 0],
  '[0, 0, 0].fill(8, 4) must return [0, 0, 0]'
);

assert.compareArray([0, 0, 0].fill(8, -1), [0, 0, 8],
  '[0, 0, 0].fill(8, -1) must return [0, 0, 8]'
);
