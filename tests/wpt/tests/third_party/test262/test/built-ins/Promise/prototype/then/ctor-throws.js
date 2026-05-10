// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `Promise.prototype.then` invoked on a constructor value that throws an
    error
es6id: 25.4.5.3
info: |
    1. Let promise be the this value.
    [...]
    3. Let C be SpeciesConstructor(promise, %Promise%).
    [...]
    5. Let resultCapability be NewPromiseCapability(C).
    6. ReturnIfAbrupt(resultCapability).

    25.4.1.5 NewPromiseCapability
    [...]
    6. Let promise be Construct(C, «executor»).
    7. ReturnIfAbrupt(promise).
features: [Symbol.species]
---*/

var BadCtor = function() {
  throw new Test262Error();
};
var originalSpecies = Object.getOwnPropertyDescriptor(Promise, Symbol.species);

Object.defineProperty(Promise, Symbol.species, {
  value: BadCtor
});

try {
  var p = new Promise(function(resolve) {
    resolve();
  });

  assert.throws(Test262Error, function() {
    p.then();
  });
} finally {
  Object.defineProperty(Promise, Symbol.species, originalSpecies);
}
