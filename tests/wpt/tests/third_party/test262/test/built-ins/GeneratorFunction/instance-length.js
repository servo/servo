// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generatorfunction
description: Definition of instance `length` property
info: |
    [...]
    3. Return CreateDynamicFunction(C, NewTarget, "generator", args).

    19.2.1.1.1 Runtime Semantics: CreateDynamicFunction

    [...]
    26. Perform FunctionInitialize(F, Normal, parameters, body, scope).
    [...]

    9.2.4 FunctionInitialize

    [...]
    3. Perform ! DefinePropertyOrThrow(F, "length",
       PropertyDescriptor{[[Value]]: len, [[Writable]]: false, [[Enumerable]]:
       false, [[Configurable]]: true}).
    [...]
includes: [propertyHelper.js]
features: [generators]
---*/

var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;

assert.sameValue(GeneratorFunction().length, 0);
assert.sameValue(GeneratorFunction('').length, 0);
assert.sameValue(GeneratorFunction('x').length, 0);
assert.sameValue(GeneratorFunction('x', '').length, 1);
assert.sameValue(GeneratorFunction('x', 'y', '').length, 2);
assert.sameValue(GeneratorFunction('x, y', '').length, 2);

verifyNotEnumerable(GeneratorFunction(), 'length');
verifyNotWritable(GeneratorFunction(), 'length');
verifyConfigurable(GeneratorFunction(), 'length');
