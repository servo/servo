// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: References to local `var` bindings resolve successfully
esid: sec-moduleevaluation
info: |
    8.1.1.5.1 GetBindingValue (N, S)

    [...]
    3. If the binding for N is an indirect binding, then
       [...]
    5. Return the value currently bound to N in envRec.


    15.2.1.16.4 ModuleDeclarationInstantiation( )

    [...]
    14. Let declaredVarNames be a new empty List.
    15. For each element d in varDeclarations do
        a. For each element dn of the BoundNames of d do
           i. If dn is not an element of declaredVarNames, then
              1. Perform ! envRec.CreateMutableBinding(dn, false).
              2. Call envRec.InitializeBinding(dn, undefined).
              3. Append dn to declaredVarNames.
    [...]

    13.3.2.4 Runtime Semantics: Evaluation

    VariableDeclaration : BindingIdentifier Initializer

    [...]
    6. Return ? PutValue(lhs, value).
flags: [module]
---*/

var varBinding = 1;
assert.sameValue(varBinding, 1);

varBinding = 2;
assert.sameValue(varBinding, 2);
