// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: >
  Reject with abrupt completion from GetIterator
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
  ...
features: [Promise.allSettled, Symbol.iterator]
flags: [async]
---*/

var poison = [];
var error = new Test262Error();
Object.defineProperty(poison, Symbol.iterator, {
  get() {
    throw error;
  }
});

try {
  Promise.allSettled(poison).then(function() {
    $DONE('The promise should be rejected, but was resolved');
  }, function(err) {
    assert.sameValue(err, error);
  }).then($DONE, $DONE);
} catch (error) {
  $DONE(`The promise should be rejected, but threw an exception: ${error.message}`);
}
