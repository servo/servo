// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Underlying iterator return is not called after result iterator observes that underlying iterator is exhausted
info: |
  Iterator.concat ( ...items )

  ...
  3. Let closure be a new Abstract Closure with no parameters that captures iterables and performs the following steps when called:
    a. For each Record iterable of iterables, do
      ...
      v. Repeat, while innerAlive is true,
        ...
        2. If innerValue is done, then
          a. Set innerAlive to false.
        ...
features: [iterator-sequencing]
---*/

let testIterator = {
  next() {
    return {
      done: true,
      value: undefined,
    };
  },
  return() {
    throw new Test262Error();
  }
};

let iterable = {
  [Symbol.iterator]() {
    return testIterator;
  }
};

let iterator = Iterator.concat(iterable);
iterator.next();
iterator.return();
