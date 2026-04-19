// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.symmetricdifference
description: Set.prototype.symmetricDifference works on subclasses of Set, but never calls the receiver's size/has/keys methods
features: [set-methods]
includes: [compareArray.js]
---*/

let sizeCount = 0;
let hasCount = 0;
let keysCount = 0;

class MySet extends Set {
  size(...rest) {
    sizeCount++;
    return super.size(...rest);
  }

  has(...rest) {
    hasCount++;
    return super.has(...rest);
  }

  keys(...rest) {
    keysCount++;
    return super.keys(...rest);
  }
}

const s1 = new MySet([1, 2]);
const s2 = new Set([2, 3]);
const expected = [1, 3];
const combined = s1.symmetricDifference(s2);

assert.compareArray([...combined], expected);
assert.sameValue(combined instanceof Set, true, "The returned object is a Set");
assert.sameValue(
  combined instanceof MySet,
  false,
  "The returned object is a Set, not a subclass"
);
assert.sameValue(sizeCount, 0, "size should not be called on the receiver");
assert.sameValue(hasCount, 0, "has should not be called on the receiver");
assert.sameValue(keysCount, 0, "keys should not be called on the receiver");
