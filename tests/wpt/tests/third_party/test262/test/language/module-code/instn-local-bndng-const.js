// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Mutable bindings are created in the lexical environment record prior to
    execution for `const` declarations
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    17. For each element d in lexDeclarations do
        a. For each element dn of the BoundNames of d do
           i. If IsConstantDeclaration of d is true, then
              1. Perform ! envRec.CreateImmutableBinding(dn, true).
    [...]
includes: [fnGlobalObject.js]
flags: [module]
---*/

var global = fnGlobalObject();

assert.throws(ReferenceError, function() {
  typeof test262;
}, 'Binding is created but not initialized.');
assert.sameValue(
  Object.getOwnPropertyDescriptor(global, 'test262'), undefined
);

const test262 = 23;

assert.sameValue(test262, 23);
assert.sameValue(
  Object.getOwnPropertyDescriptor(global, 'test262'), undefined
);

assert.throws(TypeError, function() {
  test262 = null;
});

assert.sameValue(test262, 23, 'binding is not mutable');
assert.sameValue(
  Object.getOwnPropertyDescriptor(global, 'test262'),
  undefined,
  'global binding is not effected by attempts to modify'
);
