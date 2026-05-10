// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
es6id: 18.2.1.1
description: >
  Uses the global variable envrionment of the running execution context
info: |
  [...]
  12. Let ctx be the running execution context. If direct is true, ctx will be
      the execution context that performed the direct eval. If direct is false,
      ctx will be the execution context for the invocation of the eval
      function.
  13. If direct is true, then
      [...]
  14. Else,
      a. Let lexEnv be NewDeclarativeEnvironment(evalRealm.[[GlobalEnv]]).
      b. Let varEnv be evalRealm.[[GlobalEnv]].
  [...]
  17. Let evalCxt be a new ECMAScript code execution context.
  [...]
  21. Set the evalCxt's VariableEnvironment to varEnv.
  [...]
  24. Let result be EvalDeclarationInstantiation(body, varEnv, lexEnv,
      strictEval).
features: [cross-realm]
---*/

var other = $262.createRealm().global;
var otherEval = other.eval;

otherEval('var x = 23;');
assert.sameValue(typeof x, 'undefined');
assert.sameValue(other.x, 23);
