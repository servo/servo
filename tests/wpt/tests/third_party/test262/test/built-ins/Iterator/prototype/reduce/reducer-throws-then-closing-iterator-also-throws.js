// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Attempts to close iterator when reducer throws, but that throws
info: |
  %Iterator.prototype%.reduce ( reducer )

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

let iterator = new TestIterator();

assert.throws(Test262Error, function () {
  iterator.reduce(() => {
    throw new Test262Error();
  });
});

assert.sameValue(returnCalls, 1);
