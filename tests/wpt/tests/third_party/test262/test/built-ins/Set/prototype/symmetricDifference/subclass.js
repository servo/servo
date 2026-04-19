// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.symmetricdifference
description: Set.prototype.symmetricDifference works on subclasses of Set, but returns an instance of Set
features: [set-methods]
includes: [compareArray.js]
---*/

class MySet extends Set {}

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
