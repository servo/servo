// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: -0 fromIndex becomes 0
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  5. If n ≥ 0, then
    a. Let k be n.
  ...
  7. Repeat, while k < len
  ...
features: [Array.prototype.includes]
---*/

var sample = [42, 43];
assert.sameValue(sample.includes(42, -0), true, "-0 [0]");
assert.sameValue(sample.includes(43, -0), true, "-0 [1]");
assert.sameValue(sample.includes(44, -0), false, "-0 [2]");
