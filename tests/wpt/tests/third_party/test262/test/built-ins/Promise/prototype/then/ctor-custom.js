// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.5.3
description: The constructor defined by Symbol.species takes precedence
info: |
    1. Let promise be the this value.
    2. If IsPromise(promise) is false, throw a TypeError exception.
    3. Let C be SpeciesConstructor(promise, %Promise%).
    4. ReturnIfAbrupt(C).
    5. Let resultCapability be NewPromiseCapability(C).
features: [Symbol.species, class]
---*/

var callCount = 0;
var thisValue, firstArg, argLength, getCapabilitiesExecutor;
var executor = function() {};
var p1 = new Promise(function() {});
var SpeciesConstructor = class extends Promise {
  constructor(a) {
    super(a);
    callCount += 1;
    thisValue = this;
    getCapabilitiesExecutor = a;
    argLength = arguments.length;
  }
};
var p2;

p1.constructor = function() {};
p1.constructor[Symbol.species] = SpeciesConstructor;

p2 = p1.then();

assert.sameValue(callCount, 1, 'The constructor is invoked exactly once');
assert(thisValue instanceof SpeciesConstructor);
assert.sameValue(
  argLength, 1, 'The constructor is invoked with a single argument'
);
assert.sameValue(typeof getCapabilitiesExecutor, 'function');
assert.sameValue(
  getCapabilitiesExecutor.length,
  2,
  'ES6 25.4.1.5.1: The length property of a GetCapabilitiesExecutor function is 2.'
);
assert(
  p2 instanceof SpeciesConstructor,
  'The returned object is an instance of the constructor'
);
