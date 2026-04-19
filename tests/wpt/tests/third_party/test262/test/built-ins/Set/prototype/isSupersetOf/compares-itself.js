// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
description: Set.prototype.isSupersetOf is successful when called on itself
features: [set-methods]
---*/

const s1 = new Set([1, 2]);

assert.sameValue(s1.isSupersetOf(s1), true);
