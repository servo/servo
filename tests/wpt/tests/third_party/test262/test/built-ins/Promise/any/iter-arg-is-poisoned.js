// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Promise.any(poisoned iterable) rejects with whatever error is thrown.
info: |
  Promise.any ( iterable )

  ...
  3. Let iteratorRecord be GetIterator(iterable).
  4. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
  ...

  #sec-getiterator
  GetIterator ( obj [ , hint [ , method ] ] )

  ...
  Let iterator be ? Call(method, obj).
  ...
features: [Promise.any, Symbol, Symbol.iterator, arrow-function]
flags: [async]
---*/

var poison = [];
Object.defineProperty(poison, Symbol.iterator, {
  get() {
    throw new Test262Error();
  }
});

try {
  Promise.any(poison).then(() => {
    $DONE('The promise should be rejected, but was resolved');
  }, (error) => {
    assert.sameValue(Object.getPrototypeOf(error), Test262Error.prototype);
    assert(error instanceof Test262Error);
  }).then($DONE, $DONE);
} catch (error) {
  $DONE(`The promise should be rejected, but threw an exception: ${error.message}`);
}
