// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 18.2.1.2
description: Global variables are not created if conflicting function declarations were detected.
info: |
  Runtime Semantics: EvalDeclarationInstantiation( body, varEnv, lexEnv, strict)

  ...
  8. For each d in varDeclarations, in reverse list order do
    a. If d is neither a VariableDeclaration or a ForBinding, then
      i. Assert: d is either a FunctionDeclaration or a GeneratorDeclaration.
      ii. NOTE If there are multiple FunctionDeclarations for the same name, the last declaration is used.
      iii. Let fn be the sole element of the BoundNames of d.
      iv. If fn is not an element of declaredFunctionNames, then
        1. If varEnvRec is a global Environment Record, then
          a. Let fnDefinable be varEnvRec.CanDeclareGlobalFunction(fn).
          b. ReturnIfAbrupt(fnDefinable).
          c. If fnDefinable is false, throw TypeError exception.
  ...
  15. For each String vn in declaredVarNames, in list order do
    a. If varEnvRec is a global Environment Record, then
      i. Let status be varEnvRec.CreateGlobalVarBinding(vn, true).
      ii. ReturnIfAbrupt(status).
  ...
flags: [noStrict]
---*/

try {
  eval("var shouldNotBeDefined; function NaN(){}");
} catch (e) {
  // Ignore TypeError exception.
}

assert.sameValue(Object.getOwnPropertyDescriptor(this, "shouldNotBeDefined"), undefined);
