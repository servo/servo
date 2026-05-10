// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluatebody
description: >
    The generator object is created after FunctionDeclarationInstantiation.
info: |
    14.4.10 Runtime Semantics: EvaluateBody

    1. Perform ? FunctionDeclarationInstantiation(functionObject, argumentsList).
    2. Let G be ? OrdinaryCreateFromConstructor(functionObject, "%GeneratorPrototype%",
       « [[GeneratorState]], [[GeneratorContext]] »).
    3. Perform GeneratorStart(G, FunctionBody).
    ...

features: [generators]
---*/

var g = function*(a = (g.prototype = null)) {}
var oldPrototype = g.prototype;
var it = g();

assert.notSameValue(Object.getPrototypeOf(it), oldPrototype);
