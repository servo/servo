// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.all
description: >
  Resolve when argument is a string
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
  Promise.all("").then(function(v) {
    assert.sameValue(v.length, 0);
  }, function() {
    $DONE('The promise should be resolved, but was rejected');
  }).then($DONE, $DONE);
} catch (error) {
  $DONE(`The promise should be resolved, but threw an exception: ${error.message}`);
}
