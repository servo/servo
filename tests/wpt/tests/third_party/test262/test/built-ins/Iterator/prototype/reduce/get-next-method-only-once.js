// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Gets the next method from the iterator only once
info: |
  %Iterator.prototype%.reduce ( reducer )

features: [iterator-helpers]
flags: []
---*/
let nextGets = 0;

class TestIterator extends Iterator {
  get next() {
    ++nextGets;
    let counter = 5;
    return function () {
      if (counter <= 0) {
        return { done: true, value: undefined };
      } else {
        return { done: false, value: --counter };
      }
    };
  }
}

let iterator = new TestIterator();

assert.sameValue(nextGets, 0);
assert.sameValue(
  iterator.reduce(x => x),
  4
);
assert.sameValue(nextGets, 1);
