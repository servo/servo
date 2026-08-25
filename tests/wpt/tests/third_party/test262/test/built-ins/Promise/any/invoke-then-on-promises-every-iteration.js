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

let promises = [
  Promise.resolve(),
  Promise.resolve(),
  Promise.resolve(),
];
let callCount = 0;

promises.forEach(promise => {
  let boundThen = promise.then.bind(promise);
  promise.then = function(...args) {
    assert.sameValue(this, promises[callCount]);
    callCount += 1;
    return boundThen(...args);
  };
});

Promise.any(promises)
  .then(() => {
      assert.sameValue(callCount, 3, '`then` invoked once for every iterated value');
    }).then($DONE, $DONE);
