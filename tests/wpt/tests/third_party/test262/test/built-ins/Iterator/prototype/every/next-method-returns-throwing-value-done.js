// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Underlying iterator next returns object with throwing value getter, but is already done
info: |
  %Iterator.prototype%.every ( predicate )

  4.c. Let value be ? IteratorValue(next).

features: [iterator-helpers]
flags: []
---*/
class ThrowingIterator extends Iterator {
  next() {
    return {
      done: true,
      get value() {
        throw new Test262Error();
      },
    };
  }
  return() {
    throw new Error();
  }
}

let iterator = new ThrowingIterator();
iterator.every(() => {});
