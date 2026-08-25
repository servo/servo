// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2.4.3
description: >
  Subclassed GeneratorFunction instances `prototype` property
info: |
  25.2.4.3 prototype

  Whenever a GeneratorFunction instance is created another ordinary object is
  also created and is the initial value of the generator function’s prototype
  property. The value of the prototype property is used to initialize the
  [[Prototype]] internal slot of a newly created Generator object when the
  generator function object is invoked using either [[Call]] or [[Construct]].

  This property has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: false }.
includes: [propertyHelper.js]
---*/

var GeneratorFunction = Object.getPrototypeOf(function* () {}).constructor;

class GFn extends GeneratorFunction {}

var gfn = new GFn(';');

assert.sameValue(
  Object.keys(gfn.prototype).length, 0,
  'prototype is a new ordinary object'
);
assert.sameValue(
  gfn.prototype.hasOwnProperty('constructor'), false,
  'prototype has no constructor reference'
);

verifyProperty(gfn, 'prototype', {
  writable: true,
  enumerable: false,
  configurable: false,
});
