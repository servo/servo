// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Underlying iterator has throwing next getter
info: |
  %Iterator.prototype%.flatMap ( mapper )

  4. Let iterated be ? GetIteratorDirect(O).

features: [iterator-helpers]
flags: []
---*/
class ThrowingIterator extends Iterator {
  get next() {
    throw new Test262Error();
  }
}

let iterator = new ThrowingIterator();

assert.throws(Test262Error, function () {
  iterator.flatMap(() => []);
});
