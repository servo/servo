// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Underlying iterator next returns object with throwing value getter, but is already done
info: |
  %Iterator.prototype%.drop ( limit )

  6.c.ii. If next is false, return undefined.

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

let iterator = new ThrowingIterator().drop(0);
iterator.next();

iterator = new ThrowingIterator().drop(1);
iterator.next();
