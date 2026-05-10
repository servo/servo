// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-globaldeclarationinstantiation
es6id: 15.1.8
description: >
    When multiple like-named function declarations exist, the final is assigned
    to the new binding.
info: |
  [...]
  9. Let declaredFunctionNames be a new empty List.
  10. For each d in varDeclarations, in reverse list order do
      a. If d is neither a VariableDeclaration or a ForBinding, then
         i. Assert: d is either a FunctionDeclaration or a
            GeneratorDeclaration.
         ii. NOTE If there are multiple FunctionDeclarations for the same name,
             the last declaration is used.
         iii. Let fn be the sole element of the BoundNames of d.
         iv. If fn is not an element of declaredFunctionNames, then
             1. Let fnDefinable be ? envRec.CanDeclareGlobalFunction(fn).
             2. If fnDefinable is false, throw a TypeError exception.
             3. Append fn to declaredFunctionNames.
             4. Insert d as the first element of functionsToInitialize.
  [...]
---*/

$262.evalScript(
  'function f() { return 1; }' +
  'function f() { return 2; }' +
  'function f() { return 3; }'
);

assert.sameValue(f(), 3);
