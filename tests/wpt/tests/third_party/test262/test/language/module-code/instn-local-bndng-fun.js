// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Mutable bindings are initialized in the lexical environment record prior to
    execution for function declarations
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    17. For each element d in lexDeclarations do
        a. For each element dn of the BoundNames of d do
           i. If IsConstantDeclaration of d is true, then
              [...]
           ii. Else,
               1. Perform ! envRec.CreateMutableBinding(dn, false).
           iii. If d is a GeneratorDeclaration production or a
                FunctionDeclaration production, then
                1. Let fo be the result of performing InstantiateFunctionObject
                   for d with argument env.
                2. Call envRec.InitializeBinding(dn, fo).
    [...]
includes: [fnGlobalObject.js]
flags: [module]
---*/

var global = fnGlobalObject();

assert.sameValue(typeof test262, 'function', 'function value is hoisted');
assert.sameValue(test262(), 'test262', 'hoisted function value is correct');
assert.sameValue(
  Object.getOwnPropertyDescriptor(global, 'test262'), undefined
);

test262 = null;
assert.sameValue(test262, null, 'binding is mutable');
assert.sameValue(
  Object.getOwnPropertyDescriptor(global, 'test262'), undefined
);

function test262() { return 'test262'; }

assert.sameValue(
  test262, null, 'binding is not effected by evaluation of declaration'
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(global, 'test262'),
  undefined,
  'global binding is not effected by evaluation of declaration'
);
