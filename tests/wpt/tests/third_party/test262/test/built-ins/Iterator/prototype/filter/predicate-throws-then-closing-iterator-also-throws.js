// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.filter
description: >
  Attempts to close iterator when predicate throws, but that throws
info: |
  %Iterator.prototype%.filter ( predicate )

  3.b.v. IfAbruptCloseIterator(selected, iterated).

features: [iterator-helpers]
flags: []
---*/
let returnCalls = 0;

class TestIterator extends Iterator {
  next() {
    return {
      done: false,
      value: 1,
    };
  }
  return() {
    ++returnCalls;
    throw new Error();
  }
}

let iterator = new TestIterator().filter(() => {
  throw new Test262Error();
});

assert.sameValue(returnCalls, 0);

assert.throws(Test262Error, function () {
  iterator.next();
});

assert.sameValue(returnCalls, 1);
