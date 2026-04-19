// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  Closes iterator and throws when predicate throws
info: |
  %Iterator.prototype%.some ( predicate )

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
    return {};
  }
}

let iterator = new TestIterator();

let callbackCalls = 0;

assert.throws(Test262Error, function () {
  iterator.some(() => {
    ++callbackCalls;
    throw new Test262Error();
  });
});

assert.sameValue(callbackCalls, 1);
assert.sameValue(returnCalls, 1);
