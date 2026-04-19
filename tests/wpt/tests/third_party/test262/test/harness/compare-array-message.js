// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  compareArray can handle any value in message arg
includes: [compareArray.js]
features: [BigInt, Symbol]
---*/

assert.compareArray([], [], true);
assert.compareArray([], [], 1);
assert.compareArray([], [], 1n);
assert.compareArray([], [], () => {});
assert.compareArray([], [], "test262");
assert.compareArray([], [], Symbol("1"));
assert.compareArray([], [], {});
assert.compareArray([], [], []);
assert.compareArray([], [], -1);
assert.compareArray([], [], Infinity);
assert.compareArray([], [], -Infinity);
assert.compareArray([], [], 0.1);
assert.compareArray([], [], -0.1);
