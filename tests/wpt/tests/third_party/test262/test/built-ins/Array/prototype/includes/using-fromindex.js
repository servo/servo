// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: Searches using fromIndex
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  5. If n ≥ 0, then
    a. Let k be n.
  6. Else n < 0,
    a. Let k be len + n.
    b. If k < 0, let k be 0.
  7. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
    b. If SameValueZero(searchElement, elementK) is true, return true.
    c. Increase k by 1.
  ...
features: [Array.prototype.includes]
---*/

var sample = ["a", "b", "c"];
assert.sameValue(sample.includes("a", 0), true, "includes('a', 0)");
assert.sameValue(sample.includes("a", 1), false, "includes('a', 1)");
assert.sameValue(sample.includes("a", 2), false, "includes('a', 2)");

assert.sameValue(sample.includes("b", 0), true, "includes('b', 0)");
assert.sameValue(sample.includes("b", 1), true, "includes('b', 1)");
assert.sameValue(sample.includes("b", 2), false, "includes('b', 2)");

assert.sameValue(sample.includes("c", 0), true, "includes('c', 0)");
assert.sameValue(sample.includes("c", 1), true, "includes('c', 1)");
assert.sameValue(sample.includes("c", 2), true, "includes('c', 2)");

assert.sameValue(sample.includes("a", -1), false, "includes('a', -1)");
assert.sameValue(sample.includes("a", -2), false, "includes('a', -2)");
assert.sameValue(sample.includes("a", -3), true, "includes('a', -3)");
assert.sameValue(sample.includes("a", -4), true, "includes('a', -4)");

assert.sameValue(sample.includes("b", -1), false, "includes('b', -1)");
assert.sameValue(sample.includes("b", -2), true, "includes('b', -2)");
assert.sameValue(sample.includes("b", -3), true, "includes('b', -3)");
assert.sameValue(sample.includes("b", -4), true, "includes('b', -4)");

assert.sameValue(sample.includes("c", -1), true, "includes('c', -1)");
assert.sameValue(sample.includes("c", -2), true, "includes('c', -2)");
assert.sameValue(sample.includes("c", -3), true, "includes('c', -3)");
assert.sameValue(sample.includes("c", -4), true, "includes('c', -4)");
