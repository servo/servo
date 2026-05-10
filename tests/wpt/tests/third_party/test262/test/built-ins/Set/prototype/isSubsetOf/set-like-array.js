// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issubsetof
description: Set.prototype.isSubsetOf consumes a set-like array as a set-like, not an array
features: [set-methods]
---*/

const s1 = new Set([1, 2]);
const s2 = [5, 6];
s2.size = 3;
s2.has = function (v) {
  if (v === 1) return true;
  if (v === 2) return true;
  throw new Test262Error("Set.prototype.isSubsetOf should only call its argument's has method with contents of this");
};
s2.keys = function () {
  throw new Test262Error("Set.prototype.isSubsetOf should not call its argument's keys iterator when this.size ≤ arg.size");
};

assert.sameValue(s1.isSubsetOf(s2), true);
