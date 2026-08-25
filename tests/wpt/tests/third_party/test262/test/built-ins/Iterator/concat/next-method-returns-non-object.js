// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Underlying iterator next returns non-object
info: |
  Iterator.concat ( ...items )

  ...
  3. Let closure be a new Abstract Closure with no parameters that captures iterables and performs the following steps when called:
    a. For each Record iterable of iterables, do
      ...
      v. Repeat, while innerAlive is true,
        1. Let innerValue be ? IteratorStepValue(iteratorRecord).
        ...
features: [iterator-sequencing]
---*/

let nonObjectIterator = {
  next() {
    return null;
  }
};

let iterable = {
  [Symbol.iterator]() {
    return nonObjectIterator;
  }
};

let iterator = Iterator.concat(iterable);

assert.throws(TypeError, function() {
  iterator.next();
});
