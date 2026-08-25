// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Returns a fresh iterator result object
info: |
  Iterator.concat ( ...items )

  ...
  3. Let closure be a new Abstract Closure with no parameters that captures iterables and performs the following steps when called:
    a. For each Record iterable of iterables, do
      ...
      v. Repeat, while innerAlive is true,
        1. Let innerValue be ? IteratorStepValue(iteratorRecord).
        2. If innerValue is done, then
          ...
        3. Else,
          a. Let completion be Completion(Yield(innerValue)).
    ...
features: [iterator-sequencing]
---*/

let oldIterResult = {
  done: false,
  value: 123,
};

let testIterator = {
  next() {
    return oldIterResult;
  }
};

let iterable = {
  [Symbol.iterator]() {
    return testIterator;
  }
};

let iterator = Iterator.concat(iterable);

let iterResult = iterator.next();

assert.sameValue(iterResult.done, false);
assert.sameValue(iterResult.value, 123);

assert.notSameValue(iterResult, oldIterResult);
