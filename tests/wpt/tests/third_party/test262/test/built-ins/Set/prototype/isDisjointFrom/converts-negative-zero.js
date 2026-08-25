// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.isdisjointfrom
description: Set.prototype.isDisjointFrom converts -0𝔽 to +0𝔽
features: [set-methods]
---*/

const setlikeWithMinusZero = {
  size: 1,
  has: function () {
    throw new Test262Error("Set.prototype.isDisjointFrom should not call its argument's has method when this.size > arg.size");
  },
  keys: function () {
    // we use an array here because the Set constructor would normalize away -0
    return [-0].values();
  },
};

const s1 = new Set([+0, 1]);

assert.sameValue(s1.isDisjointFrom(setlikeWithMinusZero), false);
