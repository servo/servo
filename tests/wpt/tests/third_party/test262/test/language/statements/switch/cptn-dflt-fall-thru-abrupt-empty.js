// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.12.11
description: >
    Completion value when execution continues through multiple cases and ends
    with an empty abrupt completion in the default case
info: |
    SwitchStatement : switch ( Expression ) CaseBlock

    [...]
    8. Let R be the result of performing CaseBlockEvaluation of CaseBlock with
       argument switchValue.
    9. Set the running execution context’s LexicalEnvironment to oldEnv.
    10. Return R.

    13.12.9 Runtime Semantics: CaseBlockEvaluation

    CaseBlock : { CaseClausesopt DefaultClause CaseClausesopt }

    1. Let V = undefined.
    2. Let A be the list of CaseClause items in the first CaseClauses, in
       source text order. If the first CaseClauses is not present A is « ».
    3. Let found be false.
    4. Repeat for each CaseClause C in A
       [...]
    5. Let foundInB be false.
    6. Let B be the List containing the CaseClause items in the second
       CaseClauses, in source text order. If the second CaseClauses is not
       present B is « ».
    7. If found is false, then
       [...]
    8. If foundInB is true, return NormalCompletion(V).
    9. Let R be the result of evaluating DefaultClause.
    10. If R.[[value]] is not empty, let V = R.[[value]].
    11. If R is an abrupt completion, return Completion(UpdateEmpty(R, V)).
---*/

assert.sameValue(
  eval('1; switch ("a") { case "a": 2; default: 3; break; }'),
  3,
  'Non-empty value replaces previous non-empty value'
);
assert.sameValue(
  eval('4; switch ("a") { case "a": default: 5; break; }'),
  5,
  'Non-empty value replaces empty value'
);
assert.sameValue(
  eval('6; switch ("a") { case "a": 7; default: break; }'),
  7,
  'Empty value does not replace previous non-empty value'
);

assert.sameValue(
  eval('8; do { switch ("a") { case "a": 9; default: 10; continue; } } while (false)'),
  10,
  'Non-empty value replaces previous non-empty value'
);
assert.sameValue(
  eval('11; do { switch ("a") { case "a": default: 12; continue; } } while (false)'),
  12,
  'Non-empty value replaces empty value'
);
assert.sameValue(
  eval('13; do { switch ("a") { case "a": 14; default: continue; } } while (false)'),
  14,
  'Empty value does not replace previous non-empty value'
);
