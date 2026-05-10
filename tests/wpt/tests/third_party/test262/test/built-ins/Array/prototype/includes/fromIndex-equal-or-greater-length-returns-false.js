// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: Return false if fromIndex >= ArrayLength
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  4. Let n be ? ToInteger(fromIndex). (If fromIndex is undefined, this step
  produces the value 0.)
  5. If n ≥ 0, then
    a. Let k be n.
  ...
  7. Repeat, while k < len
    ...
  8. Return false.
features: [Array.prototype.includes]
---*/

var sample = [7, 7, 7, 7];
assert.sameValue(sample.includes(7, 4), false, "length");
assert.sameValue(sample.includes(7, 5), false, "length + 1");
