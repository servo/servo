// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.12.11
description: >
    Completion value when execution continues through multiple cases and ends
    with an empty abrupt completion
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
       a. If found is false, then
          [...]
       b. If found is true, then
          i. Let R be the result of evaluating C.
          ii. If R.[[value]] is not empty, let V = R.[[value]].
          iii. If R is an abrupt completion, return Completion(UpdateEmpty(R,
               V)).
---*/

assert.sameValue(
  eval('1; switch ("a") { case "a": 2; case "b": 3; break; default: }'),
  3,
  'Non-empty value replaces previous non-empty value'
);
assert.sameValue(
  eval('4; switch ("a") { case "a": case "b": 5; break; default: }'),
  5,
  'Non-empty value replaces empty value'
);
assert.sameValue(
  eval('6; switch ("a") { case "a": 7; case "b": break; default: }'),
  7,
  'Empty value does not replace previous non-empty value'
);

assert.sameValue(
  eval('8; do { switch ("a") { case "a": 9; case "b": 10; continue; default: } } while (false)'),
  10,
  'Non-empty value replaces previous non-empty value'
);
assert.sameValue(
  eval('11; do { switch ("a") { case "a": case "b": 12; continue; default: } } while (false)'),
  12,
  'Non-empty value replaces empty value'
);
assert.sameValue(
  eval('13; do { switch ("a") { case "a": 14; case "b": continue; default: } } while (false)'),
  14,
  'Empty value does not replace previous non-empty value'
);
