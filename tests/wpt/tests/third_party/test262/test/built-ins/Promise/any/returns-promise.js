// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: Promise.any returns a Promise
info: |
  Promise.any ( iterable )

  2. Let promiseCapability be ? NewPromiseCapability(C).
  3. Let iteratorRecord be GetIterator(iterable).
  4. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).
  6. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).
  7. Return Completion(result).
features: [Promise.any]
---*/

var p = Promise.any([]);

assert(p instanceof Promise);
assert.sameValue(Object.getPrototypeOf(p), Promise.prototype);
