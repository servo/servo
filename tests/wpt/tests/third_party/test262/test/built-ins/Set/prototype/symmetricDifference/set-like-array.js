// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.symmetricdifference
description: Set.prototype.symmetricDifference consumes a set-like array as a set-like, not an array
features: [set-methods]
includes: [compareArray.js]
---*/

const s1 = new Set([1, 2]);
const s2 = [5];
s2.size = 3;
s2.has = function (v) {
  throw new Test262Error("Set.prototype.symmetricDifference should not invoke .has on its argument");
};
s2.keys = function () {
  return [2, 3, 4].values();
};

const expected = [1, 3, 4];
const combined = s1.symmetricDifference(s2);

assert.compareArray([...combined], expected);
assert.sameValue(combined instanceof Set, true, "The returned object is a Set");
