// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-generatorfunction
description: The instance created by GeneratorFunction is not a constructor
info: |
    25.2.1.1 GeneratorFunction ( p1, p2, … , pn, body )

    ...
    3. Return ? CreateDynamicFunction(C, NewTarget, "generator", args).

    19.2.1.1.1 Runtime Semantics: CreateDynamicFunction( constructor, newTarget, kind, args )

    ...
    34. If kind is "generator", then
        a. Let prototype be ObjectCreate(%GeneratorPrototype%).
        b. Perform DefinePropertyOrThrow(F, "prototype", PropertyDescriptor{[[Value]]: prototype,
            [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: false}).
    ...
features: [generators]
---*/

var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;

var instance = GeneratorFunction();

assert.throws(TypeError, function() {
  new instance();
})
