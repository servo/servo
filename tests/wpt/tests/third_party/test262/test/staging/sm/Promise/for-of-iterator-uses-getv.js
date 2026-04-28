// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - onlyStrict
description: |
  pending
esid: pending
---*/

// strict-mode to ensure |this| arguments aren't converted to objects.

var emptyIterator = {
  next() {
    return {done: true};
  }
};

Object.defineProperty(Number.prototype, Symbol.iterator, {
  configurable: true,
  get() {
    assert.sameValue(typeof this, "number");
    return function() {
      assert.sameValue(typeof this, "number");
      return emptyIterator;
    }
  }
});

Promise.all(0);
Promise.allSettled(0);
Promise.race(0);
