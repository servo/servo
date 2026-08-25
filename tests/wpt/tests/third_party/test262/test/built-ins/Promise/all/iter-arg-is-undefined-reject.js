// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.all
description: >
  Reject when argument is `undefined`
info: |
    ...
    Let iteratorRecord be GetIterator(iterable).
    IfAbruptRejectPromise(iteratorRecord, promiseCapability).
    ...

    #sec-getiterator
    GetIterator ( obj [ , hint [ , method ] ] )

    ...
    Let iterator be ? Call(method, obj).
    If Type(iterator) is not Object, throw a TypeError exception.
    ...
features: [Symbol.iterator]
flags: [async]
---*/

try {
  Promise.all(undefined).then(function() {
    $DONE('The promise should be rejected, but was resolved');
  }, function(error) {
    assert(error instanceof TypeError);
  }).then($DONE, $DONE);
} catch (error) {
  $DONE(`The promise should be rejected, but threw an exception: ${error.message}`);
}
