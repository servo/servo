// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgeneratorfunction
description: Definition of instance `length` property
info: |
    AsyncGeneratorFunction ( p1, p2, … , pn, body )
    ...
    3. Return CreateDynamicFunction(C, NewTarget, "async generator", args).

    Runtime Semantics: CreateDynamicFunction
    ...
    // the parameter "args" is sliced into "parameters" and "body"
    26. Perform FunctionInitialize(F, Normal, parameters, body, scope).
    ...

    FunctionInitialize
    ...
    2. Let len be the ExpectedArgumentCount of ParameterList.
    3. Perform ! DefinePropertyOrThrow(F, "length",
       PropertyDescriptor{[[Value]]: len, [[Writable]]: false, [[Enumerable]]:
       false, [[Configurable]]: true}).
    ...
includes: [propertyHelper.js]
features: [async-iteration]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

var instance = AsyncGeneratorFunction()

verifyProperty(AsyncGeneratorFunction(), "length", {
  value: 0,
  enumerable: false,
  writable: false,
  configurable: true,
});

assert.sameValue(AsyncGeneratorFunction('').length, 0, "test 1");
assert.sameValue(AsyncGeneratorFunction('x').length, 0, "test 2");
assert.sameValue(AsyncGeneratorFunction('x', '').length, 1, "test 3");
assert.sameValue(AsyncGeneratorFunction('x', 'y', '').length, 2, "test 4");
assert.sameValue(AsyncGeneratorFunction('x, y', '').length, 2, "test 5");


