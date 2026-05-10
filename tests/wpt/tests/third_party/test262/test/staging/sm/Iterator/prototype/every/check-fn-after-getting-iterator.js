// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
features:
  - iterator-helpers
info: |
  Iterator is not enabled unconditionally
description: |
  pending
esid: pending
---*/
const log = [];
const handlerProxy = new Proxy({}, {
  get: (target, key, receiver) => (...args) => {
    log.push(`${key}: ${args[1]?.toString()}`);
    return Reflect[key](...args);
  },
});

class TestIterator extends Iterator {
  next() {
    return {done: true};
  }
}

const iter = new Proxy(new TestIterator(), handlerProxy);
assert.throws(TypeError, () => iter.every(1));

assert.compareArray(
  log,
  ["get: every", "get: return"]
);

