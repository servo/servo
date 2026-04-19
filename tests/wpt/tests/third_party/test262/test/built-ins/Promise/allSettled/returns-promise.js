// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: Promise.allSettled returns a Promise
info: |
  Promise.allSettled ( iterable )

  3. Let promiseCapability be ? NewPromiseCapability(C).
  4. Let iteratorRecord be GetIterator(iterable).
  5. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).
  8. Return Completion(result).
features: [Promise.allSettled]
---*/

var p = Promise.allSettled([]);

assert(p instanceof Promise);
assert.sameValue(Object.getPrototypeOf(p), Promise.prototype);
