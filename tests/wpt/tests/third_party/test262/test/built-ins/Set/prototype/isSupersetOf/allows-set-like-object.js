// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
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
  size: 1,
  has: (v) => {
    throw new Test262Error("Set.prototype.isSupersetOf should not call its argument's has method");
  },
  keys: function* keys() {
    yield 1;
  },
};

assert.sameValue(s1.isSupersetOf(s2), true);
