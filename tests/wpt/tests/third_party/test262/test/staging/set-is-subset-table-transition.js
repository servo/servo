// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Test isSubsetOf set method after table transition in receiver.
features: [set-methods]
---*/

const firstSet = new Set();
firstSet.add(42);
firstSet.add(43);
firstSet.add(44);

const setLike = {
  size: 5,
  keys() {
    return [1, 2, 3, 4, 5].keys();
  },
  has(key) {
    if (key == 42) {
      // Cause a table transition in the receiver.
      firstSet.clear();
    }
    // Return true so we keep iterating the transitioned receiver.
    return true;
  }
};

assert.sameValue(firstSet.isSubsetOf(setLike), true);
assert.sameValue(firstSet.size, 0);
