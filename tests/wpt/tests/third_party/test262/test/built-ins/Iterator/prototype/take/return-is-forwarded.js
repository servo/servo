// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Underlying iterator return is called when result iterator is closed
info: |
  %Iterator.prototype%.take ( limit )

features: [iterator-helpers]
flags: []
---*/
let returnCount = 0;

class TestIterator extends Iterator {
  next() {
    return {
      done: false,
      value: 1,
    };
  }
  return() {
    ++returnCount;
    return {};
  }
}

let iterator = new TestIterator().take(0);
assert.sameValue(returnCount, 0);
iterator.return();
assert.sameValue(returnCount, 1);
iterator.return();
assert.sameValue(returnCount, 1);

returnCount = 0;

iterator = new TestIterator().take(1);
assert.sameValue(returnCount, 0);
iterator.return();
assert.sameValue(returnCount, 1);
iterator.return();
assert.sameValue(returnCount, 1);

returnCount = 0;

iterator = new TestIterator().take(1).take(1).take(1).take(1).take(1);
assert.sameValue(returnCount, 0);
iterator.return();
assert.sameValue(returnCount, 1);
iterator.return();
assert.sameValue(returnCount, 1);
