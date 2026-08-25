// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-evaldeclarationinstantiation
es6id: 18.2.1.2
description: >
    Global functions are not created if conflicting function declarations were
    detected.
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
  14. For each production f in functionsToInitialize, do
    a. Let fn be the sole element of the BoundNames of f.
    b. Let fo be the result of performing InstantiateFunctionObject for f with argument lexEnv.
    c. If varEnvRec is a global Environment Record, then
      i. Let status be varEnvRec.CreateGlobalFunctionBinding(fn, fo, true).
      ii. ReturnIfAbrupt(status).
  ...
---*/

try {
  (0,eval)("function shouldNotBeDefined1() {} function NaN() {} function shouldNotBeDefined2() {}");
} catch (e) {
  // Ignore TypeError exception.
}

assert.sameValue(
  Object.getOwnPropertyDescriptor(this, "shouldNotBeDefined1"),
  undefined,
  "declaration preceding"
);
assert.sameValue(
  Object.getOwnPropertyDescriptor(this, "shouldNotBeDefined2"),
  undefined,
  "declaration following"
);
