// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Promise.any invoked on a constructor value
esid: sec-promise.any
info: |
  2. Let promiseCapability be ? NewPromiseCapability(C).
  ...
  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).
  ...
  7. Return Completion(result).
features: [Promise.any, class]
---*/

var executor = null;
var callCount = 0;

class SubPromise extends Promise {
  constructor(a) {
    super(a);
    executor = a;
    callCount += 1;
  }
}

var instance = Promise.any.call(SubPromise, []);

assert.sameValue(instance.constructor, SubPromise);
assert.sameValue(instance instanceof SubPromise, true);

assert.sameValue(callCount, 1);
assert.sameValue(typeof executor, 'function');
