// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
es6id: 18.2.1.2
description: Throws a TypeError if a global generator function cannot be defined.
info: |
  Runtime Semantics: EvalDeclarationInstantiation( body, varEnv, lexEnv, strict)

  ...
  8. For each d in varDeclarations, in reverse list order do
    a. If d is neither a VariableDeclaration or a ForBinding, then
      i. Assert: d is either a FunctionDeclaration or a GeneratorDeclaration.
      ii. NOTE If there are multiple FunctionDeclarations for the same name,
          the last declaration is used.
      iii. Let fn be the sole element of the BoundNames of d.
      iv. If fn is not an element of declaredFunctionNames, then
        1. If varEnvRec is a global Environment Record, then
          a. Let fnDefinable be varEnvRec.CanDeclareGlobalFunction(fn).
          b. ReturnIfAbrupt(fnDefinable).
          c. If fnDefinable is false, throw TypeError exception.
        ...
features: [generators]
---*/

assert.throws(TypeError, function() {
  (0,eval)("function* NaN() {}");
});
