// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Underlying iterator's return method is called with zero arguments.
info: |
  Iterator.concat ( ...items )

  ...
  3. Let closure be a new Abstract Closure with no parameters that captures iterables and performs the following steps when called:
    a. For each Record iterable of iterables, do
      ...
      v. Repeat, while innerAlive is true,
        ...
        3. Else,
          ...
          b. If completion is an abrupt completion, then
            i. Return ? IteratorClose(iteratorRecord, completion).
features: [iterator-sequencing]
---*/

let returnCalled = 0;

let testIterator = {
  next() {
    return {done: false};
  },
  return() {
    returnCalled++;
    assert.sameValue(arguments.length, 0);
    return {done: true};
  }
};

let iterable = {
  [Symbol.iterator]() {
    return testIterator;
  }
};

let iterator;

// Call with zero arguments.
iterator = Iterator.concat(iterable);
iterator.next();
assert.sameValue(returnCalled, 0);

iterator.return();
assert.sameValue(returnCalled, 1);

// Call with one argument.
iterator = Iterator.concat(iterable);
iterator.next();
assert.sameValue(returnCalled, 1);

iterator.return(1);
assert.sameValue(returnCalled, 2);

// Call with two arguments.
iterator = Iterator.concat(iterable);
iterator.next();
assert.sameValue(returnCalled, 2);

iterator.return(1, 2);
assert.sameValue(returnCalled, 3);
