// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Reject when argument's Symbol.iterator property has the value null
info: |
  Promise.any ( iterable )

  ...
  4. Let iteratorRecord be GetIterator(iterable).
  5. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
  ...

  GetIterator ( obj [ , hint [ , method ] ] )

  ...
  3. If method is not present, then
    a. If hint is async, then
      ...
    b. Otherwise, set method to ? GetMethod(obj, @@iterator).
  4. Let iterator be ? Call(method, obj).
  5. If Type(iterator) is not Object, throw a TypeError exception.
  ...

  GetMethod

  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
  4. If IsCallable(func) is false, throw a TypeError exception.

  Call ( F, V [ , argumentsList ] )

  2. If IsCallable(F) is false, throw a TypeError exception.
features: [Promise.any, Symbol.iterator, Symbol, computed-property-names]
flags: [async]
---*/

try {
  Promise.any({
    [Symbol.iterator]: null
  }).then(function() {
    $DONE('The promise should be rejected, but was resolved');
  }, function(error) {
    assert.sameValue(Object.getPrototypeOf(error), TypeError.prototype);
    assert(error instanceof TypeError);
  }).then($DONE, $DONE);
} catch (error) {
  $DONE(`The promise should be rejected, but threw an exception: ${error.message}`);
}
