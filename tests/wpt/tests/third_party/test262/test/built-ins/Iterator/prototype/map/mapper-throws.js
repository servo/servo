// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.map
description: >
  Closes iterator and throws when mapper throws
info: |
  %Iterator.prototype%.map ( mapper )

  5.b.v. IfAbruptCloseIterator(mapped, iterated).

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

let callbackCalls = 0;
let iterator = new TestIterator().map(() => {
  ++callbackCalls;
  throw new Test262Error();
});

assert.throws(Test262Error, function () {
  iterator.next();
});

assert.sameValue(callbackCalls, 1);
assert.sameValue(returnCalls, 1);
