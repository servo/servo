// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Only one attempt is made to create a binding for any number of variable
    declarations within `for` statements
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    13. Let varDeclarations be the VarScopedDeclarations of code.
    14. Let declaredVarNames be a new empty List.
    15. For each element d in varDeclarations do
        a. For each element dn of the BoundNames of d do
           i. If dn is not an element of declaredVarNames, then
              1. Perform ! envRec.CreateMutableBinding(dn, false).
              [...]
              3. Append dn to declaredVarNames.
    [...]
flags: [module]
---*/

for (var test262; false; ) {}
for (var test262; false; ) {}
