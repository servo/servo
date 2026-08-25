// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-switch-statement-runtime-semantics-evaluation
description: Removal of lexical environment (from `case` clause)
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
var probe1, probe2;

switch (null) {
  case null:
    let x = 'inside';
    probe1 = function() { return x; };
  case null:
    probe2 = function() { return x; };
}

assert.sameValue(probe1(), 'inside', 'from first `case` clause');
assert.sameValue(probe2(), 'inside', 'from second `case` clause');
assert.sameValue(x, 'outside');
