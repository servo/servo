// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Underlying iterator's next method is called with zero arguments.
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

let nextCalled = 0;

let testIterator = {
  next() {
    nextCalled++;
    assert.sameValue(arguments.length, 0);

    return {
      done: false,
      value: 0,
    };
  }
};

let iterable = {
  [Symbol.iterator]() {
    return testIterator;
  }
};

let iterator = Iterator.concat(iterable);
assert.sameValue(nextCalled, 0);

iterator.next();
assert.sameValue(nextCalled, 1);

iterator.next(1);
assert.sameValue(nextCalled, 2);

iterator.next(1, 2);
assert.sameValue(nextCalled, 3);
