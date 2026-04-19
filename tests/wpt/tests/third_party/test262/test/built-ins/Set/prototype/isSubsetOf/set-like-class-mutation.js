// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issubsetof
description: Set.prototype.isSubsetOf behavior when a custom Set-like class mutates the receiver
features: [set-methods]
includes: [compareArray.js]
---*/

const baseSet = new Set(["a", "b", "c"]);

const evilSetLike = {
  size: 3,
  has(v) {
    if (v === "a") {
      baseSet.delete("c");
    }
    return ["x", "a", "b"].includes(v);
  },
  * keys() {
    throw new Test262Error("Set.prototype.isSubsetOf should not call its argument's keys iterator");
  },
};

const result = baseSet.isSubsetOf(evilSetLike);
assert.sameValue(result, true);

const expectedNewBase = ["a", "b"];
assert.compareArray([...baseSet], expectedNewBase);
