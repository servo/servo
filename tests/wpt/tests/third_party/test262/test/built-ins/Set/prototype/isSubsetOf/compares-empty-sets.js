// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issubsetof
description: Set.prototype.isSubsetOf can compare empty Sets
features: [set-methods]
---*/

const s1 = new Set([]);
const s2 = new Set([1, 2]);

assert.sameValue(s1.isSubsetOf(s2), true);

const s3 = new Set([1, 2]);
const s4 = new Set([]);

assert.sameValue(s3.isSubsetOf(s4), false);

const s5 = new Set([]);
const s6 = new Set([]);

assert.sameValue(s5.isSubsetOf(s6), true);
