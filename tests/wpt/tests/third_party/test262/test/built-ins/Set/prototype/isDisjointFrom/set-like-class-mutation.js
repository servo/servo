// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.isdisjointfrom
description: Set.prototype.isDisjointFrom behavior when a custom Set-like class mutates the receiver
features: [set-methods]
includes: [compareArray.js]
---*/

const baseSet = new Set(["a", "b", "c"]);

const evilSetLike = {
  size: 3,
  has(v) {
    if (v === "a") {
      baseSet.delete("b");
      baseSet.delete("c");
      baseSet.add("b");
      return false;
    }
    if (v === "b") {
      return false;
    }
    if (v === "c") {
      throw new Test262Error("Set.prototype.isDisjointFrom should not call its argument's has method with values from this which have been deleted before visiting");
    }
    throw new Test262Error("Set.prototype.isDisjointFrom should only call its argument's has method with contents of this");
  },
  * keys() {
    throw new Test262Error("Set.prototype.isDisjointFrom should not call its argument's keys iterator when this.size ≤ arg.size");
  },
};

const result = baseSet.isDisjointFrom(evilSetLike);
assert.sameValue(result, true);

const expectedNewBase = ["a", "b"];
assert.compareArray([...baseSet], expectedNewBase);
