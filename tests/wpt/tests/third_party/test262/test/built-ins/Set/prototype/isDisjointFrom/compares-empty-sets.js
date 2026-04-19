// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.isdisjointfrom
description: Set.prototype.isDisjointFrom can compare empty Sets
features: [set-methods]
---*/

const s1 = new Set([]);
const s2 = new Set([1, 2]);

assert.sameValue(s1.isDisjointFrom(s2), true);

const s3 = new Set([1, 2]);
const s4 = new Set([]);

assert.sameValue(s3.isDisjointFrom(s4), true);

const s5 = new Set([]);
const s6 = new Set([]);

assert.sameValue(s5.isDisjointFrom(s6), true);
