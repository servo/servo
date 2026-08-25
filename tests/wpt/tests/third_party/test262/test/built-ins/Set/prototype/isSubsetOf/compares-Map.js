// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issubsetof
description: Set.prototype.isSubsetOf compares with Map
features: [set-methods]
---*/

const s1 = new Set([1, 2]);
const m1 = new Map([
  [2, "two"],
  [3, "three"],
]);

assert.sameValue(s1.isSubsetOf(m1), false);
