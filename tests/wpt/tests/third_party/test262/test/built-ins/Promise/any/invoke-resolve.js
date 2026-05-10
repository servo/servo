// Copyright (C) 2019 Sergey Rubanov, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Invocation of the constructor's `resolve` method
esid: sec-promise.any
info: |
  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAny

  8. Repeat
    ...
    i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).
    ...
    r. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], rejectElement »).

flags: [async]
features: [Promise.any, arrow-function]
---*/

let boundPromiseResolve = Promise.resolve.bind(Promise);

Promise.resolve = function(...args) {
  assert.sameValue(args.length, 1, '`resolve` invoked with a single argument');
  assert.sameValue(this, Promise, '`this` value is the constructor');
  return boundPromiseResolve(...args);
};

Promise.any([1]).then(() => $DONE(), $DONE);
