// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.difference
description: Set.prototype.difference converts -0𝔽 to +0𝔽
info: |
    7.b.ii If nextValue is -0𝔽, set nextValue to +0𝔽.
features: [set-methods]
includes: [compareArray.js]
---*/

const setlikeWithMinusZero = {
  size: 1,
  has: function () {
    throw new Test262Error("Set.prototype.difference should not call its argument's has method when this.size > arg.size");
  },
  keys: function () {
    // we use an array here because the Set constructor would normalize away -0
    return [-0].values();
  },
};

const s1 = new Set([+0, 1]);
let expected = [1];
let combined = s1.difference(setlikeWithMinusZero);

assert.compareArray([...combined], expected);
assert.sameValue(combined instanceof Set, true, "The returned object is a Set");
