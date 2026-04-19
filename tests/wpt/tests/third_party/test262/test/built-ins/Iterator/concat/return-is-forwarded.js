// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Underlying iterator return is called when result iterator is closed
info: |
  Iterator.concat ( ...items )

  ...
  3. Let closure be a new Abstract Closure with no parameters that captures iterables and performs the following steps when called:
    a. For each Record iterable of iterables, do
      ...
      v. Repeat, while innerAlive is true,
        ...
        3. Else,
          a. Let completion be Completion(Yield(innerValue)).
          b. If completion is an abrupt completion, then
            i. Return ? IteratorClose(iteratorRecord, completion).
    ...
features: [iterator-sequencing]
---*/

let returnCount = 0;

let testIterator = {
  next() {
    return {
      done: false,
      value: 1,
    };
  },
  return() {
    ++returnCount;
    return {};
  }
};

let iterable = {
  [Symbol.iterator]() {
    return testIterator;
  }
};

let iterator = Iterator.concat(iterable);
assert.sameValue(returnCount, 0);

let iterResult = iterator.next();
assert.sameValue(returnCount, 0);
assert.sameValue(iterResult.done, false);
assert.sameValue(iterResult.value, 1);

iterator.return();
assert.sameValue(returnCount, 1);

iterator.return();
assert.sameValue(returnCount, 1);
