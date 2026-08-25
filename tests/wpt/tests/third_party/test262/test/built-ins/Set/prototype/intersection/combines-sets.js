// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.intersection
description: Set.prototype.intersection combines Sets
features: [set-methods]
includes: [compareArray.js]
---*/

const s1 = new Set([1, 2]);
const s2 = new Set([2, 3]);
const expected = [2];
const combined = s1.intersection(s2);

assert.compareArray([...combined], expected);
assert.sameValue(combined instanceof Set, true, "The returned object is a Set");
