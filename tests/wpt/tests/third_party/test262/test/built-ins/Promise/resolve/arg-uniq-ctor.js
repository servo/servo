// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `Promise.resolve` invoked with a Promise with a unique constructor
es6id: 25.4.4.5
info: |
    1. Let C be the this value.
    [...]
    3. If IsPromise(x) is true,
       a. Let xConstructor be Get(x, "constructor").
       b. ReturnIfAbrupt(xConstructor).
       c. If SameValue(xConstructor, C) is true, return x.
    4. Let promiseCapability be NewPromiseCapability(C).
    [...]
    8. Return promiseCapability.[[Promise]].
---*/

var promise1 = new Promise(function() {});
var promise2;

promise1.constructor = null;

promise2 = Promise.resolve(promise1);

assert.sameValue(promise1 === promise2, false);
