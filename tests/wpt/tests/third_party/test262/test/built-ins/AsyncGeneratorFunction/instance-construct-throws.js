// Copyright 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgeneratorfunction
description: The instance created by AsyncGeneratorFunction is not a constructor
info: |
    AsyncGeneratorFunction ( p1, p2, … , pn, body )
    ...
    3. Return ? CreateDynamicFunction(C, NewTarget, "async generator", args).

    Runtime Semantics: CreateDynamicFunction( constructor, newTarget, kind, args )
    ...
    32. Let F be FunctionAllocate(proto, strict, kind).
    ...

    FunctionAllocate ( functionPrototype, strict, functionKind )
    // [[Construct]] and [[ConstructKind]] are not set for functionKind="async generators"

features: [async-iteration]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

var instance = AsyncGeneratorFunction();

assert.throws(TypeError, function() {
    new instance();
})


