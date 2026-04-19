// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-asyncgeneratorfunction-prototype
description: >
  %AsyncGeneratorFunction.prototype% is an ordinary non-callable object.
info: |
  Properties of the AsyncGeneratorFunction Prototype Object

  The AsyncGeneratorFunction prototype object:

  [...]
  * is an ordinary object.
  * is not a function object and does not have an [[ECMAScriptCode]] internal slot
    or any other of the internal slots listed in Table 28 or Table 75.
features: [async-iteration]
---*/

var AsyncGeneratorFunctionPrototype = Object.getPrototypeOf(async function* () {});

assert.sameValue(typeof AsyncGeneratorFunctionPrototype, "object");
assert.throws(TypeError, function() {
  AsyncGeneratorFunctionPrototype();
});

assert(!AsyncGeneratorFunctionPrototype.hasOwnProperty("length"), "length");
assert(!AsyncGeneratorFunctionPrototype.hasOwnProperty("name"), "name");
