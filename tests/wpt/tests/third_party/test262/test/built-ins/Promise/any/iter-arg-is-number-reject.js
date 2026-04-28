// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Promise.any(number) rejects with TypeError.
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
  If Type(iterator) is not Object, throw a TypeError exception.
  ...
features: [Promise.any]
flags: [async]
---*/

try {
  Promise.any(1).then(function() {
    $DONE('The promise should be rejected, but was resolved');
  }, function(error) {
    assert.sameValue(Object.getPrototypeOf(error), TypeError.prototype);
    assert(error instanceof TypeError);
  }).then($DONE, $DONE);
} catch (error) {
  $DONE(`The promise should be rejected, but threw an exception: ${error.message}`);
}
