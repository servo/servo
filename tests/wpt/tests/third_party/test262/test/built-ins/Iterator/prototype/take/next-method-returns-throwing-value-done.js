// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Underlying iterator next returns object with throwing value getter, but is already done
info: |
  %Iterator.prototype%.take ( limit )

features: [iterator-helpers]
flags: []
---*/
class ReturnCalledError extends Error {}
class ValueGetterError extends Error {}

class ThrowingIterator extends Iterator {
  next() {
    return {
      done: true,
      get value() {
        throw new ValueGetterError();
      },
    };
  }
  return() {
    throw new ReturnCalledError();
  }
}

let iterator = new ThrowingIterator().take(0);
assert.throws(ReturnCalledError, function () {
  iterator.next();
});

iterator = new ThrowingIterator().take(1);
iterator.next();
