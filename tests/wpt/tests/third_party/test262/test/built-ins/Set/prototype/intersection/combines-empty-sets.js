// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.intersection
description: Set.prototype.intersection can combine empty Sets
features: [set-methods]
includes: [compareArray.js]
---*/

const s1 = new Set([]);
const s2 = new Set([1, 2]);
let expected = [];
let combined = s1.intersection(s2);

assert.compareArray([...combined], expected);
assert.sameValue(combined instanceof Set, true, "The returned object is a Set");

const s3 = new Set([1, 2]);
const s4 = new Set([]);
expected = [];
combined = s3.intersection(s4);

assert.compareArray([...combined], expected);
assert.sameValue(combined instanceof Set, true, "The returned object is a Set");

const s5 = new Set([]);
const s6 = new Set([]);
expected = [];
combined = s5.intersection(s6);

assert.compareArray([...combined], expected);
assert.sameValue(combined instanceof Set, true, "The returned object is a Set");
