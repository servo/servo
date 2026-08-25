// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Underlying iterator has throwing next getter
info: |
  %Iterator.prototype%.take ( limit )

  7. Let iterated be ? GetIteratorDirect(this value).

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
  iterator.take(0);
});
