// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.5.3
description: >
    A TypeError is thrown when the `this` value has a non-object `constructor` property
info: |
    1. Let promise be the this value.
    2. If IsPromise(promise) is false, throw a TypeError exception.
    3. Let C be SpeciesConstructor(promise, %Promise%).
    4. ReturnIfAbrupt(C).
    5. Let resultCapability be NewPromiseCapability(C).
---*/

var p = new Promise(function() {});
p.constructor = null;

assert.throws(TypeError, function() {
  p.then();
});
