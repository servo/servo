// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Underlying iterator return is not called before initial call to next method
features: [iterator-sequencing]
---*/

let testIterator = {
  next() {
    throw new Test262Error();
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
iterator.return();
iterator.next();
iterator.return();
