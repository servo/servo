// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issubsetof
description: GetSetRecord allows Set-like objects
info: |
    1. If obj is not an Object, throw a TypeError exception.
    2. Let rawSize be ? Get(obj, "size").
    ...
    7. Let has be ? Get(obj, "has").
    ...
    9. Let keys be ? Get(obj, "keys").
features: [set-methods]
---*/

const s1 = new Set([1, 2]);
const s2 = {
  size: 2,
  has: (v) => {
    if (v === 1) return false;
    if (v === 2) return true;
    throw new Test262Error("Set.prototype.isSubsetOf should only call its argument's has method with contents of this");
  },
  keys: function* keys() {
    throw new Test262Error("Set.prototype.isSubsetOf should not call its argument's keys iterator");
  },
};

assert.sameValue(s1.isSubsetOf(s2), false);
