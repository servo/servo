// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.map
description: >
  Iterator has throwing return
info: |
  %Iterator.prototype%.map ( mapper )

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
  return() {
    throw new Test262Error();
  }
}

let iterator = new IteratorThrows().map(() => 0);

assert.throws(Test262Error, function () {
  iterator.return();
});
