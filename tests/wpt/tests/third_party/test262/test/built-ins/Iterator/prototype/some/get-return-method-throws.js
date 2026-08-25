// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  Iterator has throwing return getter
info: |
  %Iterator.prototype%.some ( predicate )

features: [iterator-helpers]
flags: []
---*/
class IteratorThrows extends Iterator {
  next() {
    return {
      done: false,
      value: 0,
    };
  }
  get return() {
    throw new Test262Error();
  }
}

let iterator = new IteratorThrows([1, 2]);

assert.throws(Test262Error, function () {
  iterator.some(() => true);
});
