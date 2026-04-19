// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
description: Set.prototype.isSupersetOf can compare Sets that have the same content
features: [set-methods]
---*/

const s1 = new Set([1, 2]);
const s2 = new Set([1, 2]);

assert.sameValue(s1.isSupersetOf(s2), true);
