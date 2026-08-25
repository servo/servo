// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-async-function-instances
description: >
  Async function instances are not constructors and do not have a
  [[Construct]] slot.
info: |
    25.5.1.1 AsyncFunction( p1, p2, … , pn, body )

    ...
    3. Return CreateDynamicFunction(C, NewTarget, "async", args).

    19.2.1.1.1 Runtime Semantics: CreateDynamicFunction( constructor, newTarget, kind, args )

    ...
    33. Perform FunctionInitialize(F, Normal, parameters, body, scope).
    34. If kind is "generator", then
        ...
    35. Else if kind is "normal", perform MakeConstructor(F).
    36. NOTE: Async functions are not constructable and do not have a [[Construct]] internal method
        or a  "prototype" property.
    ...
---*/

async function foo() {}
assert.throws(TypeError, function() {
  new foo();
});

var AsyncFunction = Object.getPrototypeOf(foo).constructor;
var instance = AsyncFunction();

assert.throws(TypeError, function() {
  new instance();
})
