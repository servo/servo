// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-switch-statement-runtime-semantics-evaluation
description: Creation of new lexical environment (into `case` clause)
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
features: [let]
---*/

let x = 'outside';
var probeExpr, probeSelector, probeStmt;

switch (probeExpr = function() { return x; }, null) {
  case probeSelector = function() { return x; }, null:
    probeStmt = function() { return x; };
    let x = 'inside';
}

assert.sameValue(probeExpr(), 'outside');
assert.sameValue(
  probeSelector(), 'inside', 'reference from "selector" Expression'
);
assert.sameValue(probeStmt(), 'inside', 'reference from Statement position');
