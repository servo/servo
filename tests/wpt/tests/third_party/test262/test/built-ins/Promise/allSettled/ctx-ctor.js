// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Promise.allSettled invoked on a constructor value
esid: sec-promise.allsettled
info: |
  3. Let promiseCapability be ? NewPromiseCapability(C).
  ...
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  ...
  8. Return Completion(result).
features: [Promise.allSettled, class]
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

var instance = Promise.allSettled.call(SubPromise, []);

assert.sameValue(instance.constructor, SubPromise);
assert.sameValue(instance instanceof SubPromise, true);

assert.sameValue(callCount, 1);
assert.sameValue(typeof executor, 'function');
