// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  Iterator has throwing next getter
info: |
  %Iterator.prototype%.some ( predicate )

features: [iterator-helpers]
flags: []
---*/
class IteratorThrows extends Iterator {
  get next() {
    throw new Test262Error();
  }
}

let iterator = new IteratorThrows();

assert.throws(Test262Error, function () {
  iterator.some(() => {});
});
