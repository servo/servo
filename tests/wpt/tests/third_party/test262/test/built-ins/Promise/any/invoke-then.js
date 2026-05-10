// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Invocation of the instance's `then` method
esid: sec-promise.any
info: |
  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).
  6. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAny

  r. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], rejectElement »).

flags: [async]
features: [Promise.any, arrow-function]
---*/

let promise = Promise.resolve();
let boundThen = promise.then.bind(promise);
let callCount = 0;

promise.then = function(resolver, rejectElement) {
  assert.sameValue(this, promise);
  assert.sameValue(typeof resolver, 'function');
  assert.sameValue(resolver.length, 1, 'resolver.length is 1');
  assert.sameValue(typeof rejectElement, 'function');
  assert.sameValue(rejectElement.length, 1, 'rejectElement.length is 0');
  callCount++;
  return boundThen(resolver, rejectElement);
};

Promise.any([promise]).then(() => {
  assert.sameValue(callCount, 1);
  $DONE();
}, $DONE);
