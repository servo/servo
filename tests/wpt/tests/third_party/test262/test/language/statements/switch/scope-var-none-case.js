// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-switch-statement-runtime-semantics-evaluation
description: Retainment of existing variable environment (`case` clause)
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

var probeExpr, probeSelector, probeStmt;
var probeBefore = function() { return x; };

switch (eval('var x = 1;'), probeExpr = function() { return x; }, null) {
  case eval('var x = 2;'), probeSelector = function() { return x; }, null:
    probeStmt = function() { return x; };
    var x = 3;
}

assert.sameValue(probeBefore(), 3, 'reference preceding statement');
assert.sameValue(probeExpr(), 3, 'reference from first Expression');
assert.sameValue(probeSelector(), 3, 'reference from "selector" Expression');
assert.sameValue(probeStmt(), 3, 'reference from Statement position');
assert.sameValue(x, 3, 'reference following statement');
