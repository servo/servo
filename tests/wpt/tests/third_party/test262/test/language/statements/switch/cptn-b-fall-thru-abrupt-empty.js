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
       [...]
    5. Let foundInB be false.
    6. Let B be the List containing the CaseClause items in the second
       CaseClauses, in source text order. If the second CaseClauses is not
       present B is « ».
    7. If found is false, then
       a. Repeat for each CaseClause C in B
          i. If foundInB is false, then
             1. Let clauseSelector be the result of CaseSelectorEvaluation of
                C.
             2. If clauseSelector is an abrupt completion, then
                a. If clauseSelector.[[value]] is empty, return
                   Completion{[[type]]: clauseSelector.[[type]], [[value]]:
                   undefined, [[target]]: clauseSelector.[[target]]}.
                b. Else, return Completion(clauseSelector).
             3. Let foundInB be the result of performing Strict Equality
                Comparison input === clauseSelector.[[value]].
           ii. If foundInB is true, then
               1. Let R be the result of evaluating CaseClause C.
               2. If R.[[value]] is not empty, let V = R.[[value]].
               3. If R is an abrupt completion, return
                  Completion(UpdateEmpty(R, V)).
---*/

assert.sameValue(
  eval('1; switch ("a") { default: case "a": 2; case "b": 3; break; }'),
  3,
  'Non-empty value replaces previous non-empty value'
);
assert.sameValue(
  eval('4; switch ("a") { default: case "a": case "b": 5; break; }'),
  5,
  'Non-empty value replaces empty value'
);
assert.sameValue(
  eval('6; switch ("a") { default: case "a": 7; case "b": break; }'),
  7,
  'Empty value does not replace previous non-empty value'
);

assert.sameValue(
  eval('8; do { switch ("a") { default: case "a": 9; case "b": 10; continue; } } while (false)'),
  10,
  'Non-empty value replaces previous non-empty value'
);
assert.sameValue(
  eval('11; do { switch ("a") { default: case "a": case "b": 12; continue; } } while (false)'),
  12,
  'Non-empty value replaces empty value'
);
assert.sameValue(
  eval('13; do { switch ("a") { default: case "a": 14; case "b": continue; } } while (false)'),
  14,
  'Empty value does not replace previous non-empty value'
);
