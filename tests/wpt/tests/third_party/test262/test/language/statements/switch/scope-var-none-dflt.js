// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-switch-statement-runtime-semantics-evaluation
description: Retainment of existing variable environment (`default` clause)
info: |
    1. Let exprRef be the result of evaluating Expression.
    2. Let switchValue be ? GetValue(exprRef).
    3. Let oldEnv be the running execution context's LexicalEnvironment.
    4. Let blockEnv be NewDeclarativeEnvironment(oldEnv).
    5. Perform BlockDeclarationInstantiation(CaseBlock, blockEnv).
    6. Set the running execution context's LexicalEnvironment to blockEnv.
    7. Let R be the result of performing CaseBlockEvaluation of CaseBlock with
      argument switchValue.
    [...]
flags: [noStrict]
---*/

var probeExpr, probeStmt;
var probeBefore = function() { return x; };

switch (eval('var x = 1;'), probeExpr = function() { return x; }) {
  default:
    probeStmt = function() { return x; };
    var x = 2;
}

assert.sameValue(probeBefore(), 2, 'reference preceding statment');
assert.sameValue(probeExpr(), 2, 'reference from Expression position');
assert.sameValue(probeStmt(), 2, 'reference from Statement position');
assert.sameValue(x, 2, 'reference following statement');
