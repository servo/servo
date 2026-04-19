// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `Promise.reject` invoked on a constructor value
es6id: 25.4.4.5
info: |
    1. Let C be the this value.
    [...]
    3. Let promiseCapability be NewPromiseCapability(C).
    [...]
    7. Return promiseCapability.[[Promise]].
features: [class]
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

var instance = Promise.reject.call(SubPromise);

assert.sameValue(instance.constructor, SubPromise);
assert.sameValue(instance instanceof SubPromise, true);

assert.sameValue(callCount, 1);
assert.sameValue(typeof executor, 'function');
