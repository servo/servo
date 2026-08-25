// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: handle Infinity values for fromIndex
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  4. Let n be ? ToInteger(fromIndex). (If fromIndex is undefined, this step
  produces the value 0.)
  5. If n ≥ 0, then
    a. Let k be n.
  6. Else n < 0,
    a. Let k be len + n.
    b. If k < 0, let k be 0.
  7. Repeat, while k < len
    ...
  8. Return false.
features: [Array.prototype.includes]
---*/

var sample = [42, 43, 43, 41];

assert.sameValue(
  sample.includes(43, Infinity),
  false,
  "includes(43, Infinity)"
);
assert.sameValue(
  sample.includes(43, -Infinity),
  true,
  "includes(43, -Infinity)"
);
