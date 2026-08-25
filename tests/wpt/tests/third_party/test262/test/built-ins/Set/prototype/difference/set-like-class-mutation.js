// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.difference
description: Set.prototype.difference maintains values even when a custom Set-like class mutates the receiver
features: [set-methods]
includes: [compareArray.js]
---*/

const baseSet = new Set(["a", "b", "c", "d", "e"]);

function mutatingIterator() {
  let index = 0;
  let values = ["x", "b", "b"];
  return {
    next() {
      if (index === 0) {
        baseSet.delete("b");
        baseSet.delete("c");
        baseSet.add("b");
        baseSet.add("d");
      }
      return {
        done: index >= values.length,
        value: values[index++],
      };
    },
  };
}

const evilSetLike = {
  size: 3,
  get has() {
    baseSet.add("q");
    return function () {
      throw new Test262Error("Set.prototype.difference should not invoke .has on its argument when this.size > other.size");
    };
  },
  keys() {
    return mutatingIterator();
  },
};

const combined = baseSet.difference(evilSetLike);
const expectedCombined = ["a", "c", "d", "e", "q"];
assert.compareArray([...combined], expectedCombined);

const expectedNewBase = ["a", "d", "e", "q", "b"];
assert.compareArray([...baseSet], expectedNewBase);
