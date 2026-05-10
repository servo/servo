// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgeneratorfunction
description: Definition of instance `prototype` property
info: |
    AsyncGeneratorFunction ( p1, p2, … , pn, body )
    ...
    3. Return CreateDynamicFunction(C, NewTarget, "async generator", args).

    Runtime Semantics: CreateDynamicFunction
    ...
    37. Else if kind is "async generator", then
        a. Let prototype be ObjectCreate(%AsyncGeneratorPrototype%).
        b. Perform DefinePropertyOrThrow(F, "prototype",
           PropertyDescriptor{[[Value]]: prototype, [[Writable]]: true,
           [[Enumerable]]: false, [[Configurable]]: false}).
    ...
includes: [propertyHelper.js]
features: [async-iteration]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

var instance = AsyncGeneratorFunction();

assert.sameValue(typeof instance.prototype, 'object');
assert.sameValue(
  Object.getPrototypeOf(instance.prototype),
  Object.getPrototypeOf(instance).prototype
);

verifyProperty(instance, "prototype", {
  enumerable: false,
  writable: true,
  configurable: false,
});
