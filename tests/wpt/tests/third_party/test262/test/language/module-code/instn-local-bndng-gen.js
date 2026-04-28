// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Mutable bindings are initialized in the lexical environment record prior to
    execution for generator function declarations
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    15. For each element d in varDeclarations do
        a. For each element dn of the BoundNames of d do
           i. If dn is not an element of declaredVarNames, then
              1. Perform ! envRec.CreateMutableBinding(dn, false).
              2. Call envRec.InitializeBinding(dn, undefined).
              3. Append dn to declaredVarNames.
    [...]
includes: [fnGlobalObject.js]
flags: [module]
---*/

var global = fnGlobalObject();

assert.sameValue(
  typeof test262, 'function', 'generator function value is hoisted'
);
assert.sameValue(
  test262().next().value,
  'test262',
  'hoisted generator function value is correct'
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(global, 'test262'), undefined
);

test262 = null;
assert.sameValue(test262, null, 'binding is mutable');
assert.sameValue(
  Object.getOwnPropertyDescriptor(global, 'test262'), undefined
);

function* test262() { return 'test262'; }

assert.sameValue(
  test262, null, 'binding is not effected by evaluation of declaration'
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(global, 'test262'),
  undefined,
  'global binding is not effected by evaluation of declaration'
);
