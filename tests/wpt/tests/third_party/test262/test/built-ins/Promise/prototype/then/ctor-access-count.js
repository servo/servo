// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.5.3
description: The instance's `constructor` property is accessed exactly once
info: |
    1. Let promise be the this value.
    2. If IsPromise(promise) is false, throw a TypeError exception.
    3. Let C be SpeciesConstructor(promise, %Promise%).
    4. ReturnIfAbrupt(C).
    5. Let resultCapability be NewPromiseCapability(C).
    6. ReturnIfAbrupt(resultCapability).
    7. Return PerformPromiseThen(promise, onFulfilled, onRejected,
       resultCapability).

    7.3.20 SpeciesConstructor ( O, defaultConstructor )

    1. Assert: Type(O) is Object.
    2. Let C be Get(O, "constructor").
    3. ReturnIfAbrupt(C).
    4. If C is undefined, return defaultConstructor.
    5. If Type(C) is not Object, throw a TypeError exception.
    6. Let S be Get(C, @@species).
    7. ReturnIfAbrupt(S).
    8. If S is either undefined or null, return defaultConstructor.
    9. If IsConstructor(S) is true, return S.
    10. Throw a TypeError exception.
flags: [async]
---*/

var callCount = 0;
var prms = new Promise(function(resolve) {
  resolve();
});
Object.defineProperty(prms, 'constructor', {
  get: function() {
    callCount += 1;
    return Promise;
  }
});

prms.then(function() {
  if (callCount !== 1) {
    $DONE('Expected constructor access count: 1. Actual: ' + callCount);
    return;
  }

  $DONE();
}, function() {
  $DONE('The promise should not be rejected.');
});
