// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.isdisjointfrom
description: GetSetRecord allows instances of Set-like classes
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
const s2 = new class {
  get size() {
    return 2;
  }
  has(v) {
    if (v === 1 || v === 2) return false;
    throw new Test262Error("Set.prototype.isDisjointFrom should only call its argument's has method with contents of this");
  }
  * keys() {
    throw new Test262Error("Set.prototype.isDisjointFrom should not call its argument's keys iterator when this.size ≤ arg.size");
  }
};

assert.sameValue(s1.isDisjointFrom(s2), true);
