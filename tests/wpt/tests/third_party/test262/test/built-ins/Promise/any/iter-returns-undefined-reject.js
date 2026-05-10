// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Reject when argument's Symbol.iterator returns undefined
info: |
    ...
    Let iteratorRecord be GetIterator(iterable).
    IfAbruptRejectPromise(iteratorRecord, promiseCapability).
    ...

    GetIterator ( obj [ , hint [ , method ] ] )

    ...
    Let iterator be ? Call(method, obj).
    If Type(iterator) is not Object, throw a TypeError exception.
    ...
features: [Promise.any, Symbol.iterator]
flags: [async]
---*/

let callCount = 0;
Promise.any({
  [Symbol.iterator]() {
    callCount++;
    return undefined;
  }
}).then(() => {
  $DONE('The promise should be rejected, but was resolved');
}, (error) => {
  assert.sameValue(callCount, 1, 'callCount === 1');
  assert(error instanceof TypeError);
}).then($DONE, $DONE);
