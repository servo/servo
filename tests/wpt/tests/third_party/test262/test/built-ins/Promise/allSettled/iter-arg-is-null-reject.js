// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: >
  Reject when argument is `null`
info: |
  Promise.allSettled ( iterable )

  ...
  4. Let iteratorRecord be GetIterator(iterable).
  5. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
  ...

  #sec-getiterator
  GetIterator ( obj [ , hint [ , method ] ] )

  ...
  Let iterator be ? Call(method, obj).
  If Type(iterator) is not Object, throw a TypeError exception.
  ...
features: [Promise.allSettled, Symbol.iterator]
flags: [async]
---*/

try {
  Promise.allSettled(null).then(function() {
    $DONE('The promise should be rejected, but was resolved');
  }, function(error) {
    assert.sameValue(Object.getPrototypeOf(error), TypeError.prototype);
    assert(error instanceof TypeError);
  }).then($DONE, $DONE);
} catch (error) {
  $DONE(`The promise should be rejected, but threw an exception: ${error.message}`);
}
