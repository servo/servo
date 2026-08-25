// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
description: Set.prototype.isSupersetOf can compare empty Sets
features: [set-methods]
---*/

const s1 = new Set([]);
const s2 = new Set([1, 2]);

assert.sameValue(s1.isSupersetOf(s2), false);

const s3 = new Set([1, 2]);
const s4 = new Set([]);

assert.sameValue(s3.isSupersetOf(s4), true);

const s5 = new Set([]);
const s6 = new Set([]);

assert.sameValue(s5.isSupersetOf(s6), true);
