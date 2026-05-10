// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Promise.all() does not retrieve `Symbol.species` property of the `this` value
es6id: 25.4.4.1
info: |
    1. Let C be the this value.
    2. If Type(C) is not Object, throw a TypeError exception.
    3. Let promiseCapability be ? NewPromiseCapability(C).
    ...
features: [Symbol.species]
---*/

function C(executor) {
  executor(function() {}, function() {});
}

C.resolve = function() {};
Object.defineProperty(C, Symbol.species, {
  get: function() {
    throw new Test262Error("Getter for Symbol.species called");
  }
});

Promise.all.call(C, []);
