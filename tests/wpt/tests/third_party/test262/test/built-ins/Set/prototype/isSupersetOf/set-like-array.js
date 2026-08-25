// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
description: Set.prototype.isSupersetOf consumes a set-like array as a set-like, not an array
features: [set-methods]
---*/

const s1 = new Set([1, 2]);
const s2 = [1];
s2.size = 3;
s2.has = function (v) {
  if (v === 1) return true;
  if (v === 2) return true;
  throw new Test262Error("Set.prototype.isSupersetOf should only call its argument's has method with contents of this");
};
s2.keys = function () {
  throw new Test262Error("Set.prototype.isSupersetOf should not call its argument's keys iterator when this.size ≤ arg.size");
};

assert.sameValue(s1.isSupersetOf(s2), false);
