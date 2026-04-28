// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-switch-statement-runtime-semantics-evaluation
description: Removal of lexical environment (from `default` clause)
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
var probeDefault, probeDefaultBeforeCase, probeCase;

switch (null) {
  default:
    let x = 'inside';
    probeDefault = function() { return x; };
}

assert.sameValue(probeDefault(), 'inside', 'from lone `default` clause`');
assert.sameValue(x, 'outside');

switch (null) {
  default:
    let x = 'inside';
    probeDefaultBeforeCase = function() { return x; };
  case 0:
    probeCase = function() { return x; };
}

assert.sameValue(
  probeDefaultBeforeCase(),
  'inside',
  'from `default` clause preceding `case` clause'
);
assert.sameValue(
  probeCase(), 'inside', 'from `case` clause following `default` clause'
);
