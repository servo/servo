// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
description: Precedence of final declaration when bindings are duplicated
info: |
    [...]
    8. For each d in varDeclarations, in reverse list order do
       a. If d is neither a VariableDeclaration or a ForBinding, then
          i. Assert: d is either a FunctionDeclaration or a
             GeneratorDeclaration.
          [...]
          iv. If fn is not an element of declaredFunctionNames, then
              [...]
              3. Insert d as the first element of functionsToInitialize.
    [...]
    15. For each production f in functionsToInitialize, do
        a. Let fn be the sole element of the BoundNames of f.
        b. Let fo be the result of performing InstantiateFunctionObject for f
           with argument lexEnv.
    [...]
flags: [noStrict]
---*/

var initial;

eval('initial = f; function f() { return "first"; } function f() { return "second"; }');

assert.sameValue(initial(), 'second', 'initial value');
assert.sameValue(f(), 'second', 'value following declaration evaluation');
