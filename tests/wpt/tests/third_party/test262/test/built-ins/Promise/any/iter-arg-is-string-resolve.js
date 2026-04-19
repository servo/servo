// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Promise.any('non-empty-string') resolves with the first character in the non-empty string
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
features: [Promise.any, arrow-function]
flags: [async]
---*/

try {
  Promise.any('xyz').then(v => {
    assert.sameValue(v, 'x');
    assert.sameValue(v.length, 1);
  }, error => {
    $DONE(`The promise should be resolved, but was rejected with error: ${error.message}`);
  }).then($DONE, $DONE);
} catch (error) {
  $DONE(`The promise should be resolved, but threw an exception: ${error.message}`);
}
