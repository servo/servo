// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
description: Set.prototype.isSupersetOf behavior when a custom Set-like class mutates the receiver
features: [set-methods]
includes: [compareArray.js]
---*/

const baseSet = new Set(["a", "b", "c"]);

const evilSetLike = {
  size: 3,
  has(v) {
    throw new Test262Error("Set.prototype.isSupersetOf should not call its argument's has method");
  },
  * keys() {
    yield "a";
    baseSet.delete("b");
    baseSet.delete("c");
    baseSet.add("b");
    yield "b";
  },
};

const result = baseSet.isSupersetOf(evilSetLike);
assert.sameValue(result, true);

const expectedNewBase = ["a", "b"];
assert.compareArray([...baseSet], expectedNewBase);
