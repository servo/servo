// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  `take` and `drop` throw eagerly when passed values that can't be converted to numbers.
info: |
  Iterator Helpers proposal 2.1.5.4 and 2.1.5.5
features:
  - iterator-helpers
---*/

//
//
const iter = [].values();
const methods = [
  value => iter.take(value),
  value => iter.drop(value),
];

const objectWithToPrimitive = {
  [Symbol.toPrimitive]() {
    return {};
  }
};

for (const method of methods) {
  assert.throws(TypeError, () => method(0n));
  assert.throws(TypeError, () => method(Symbol('')));
  assert.throws(TypeError, () => method(objectWithToPrimitive));
}

