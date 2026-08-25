// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.12.11
description: Completion value when the final case matches
info: |
    SwitchStatement : switch ( Expression ) CaseBlock

    [...]
    8. Let R be the result of performing CaseBlockEvaluation of CaseBlock with
       argument switchValue.
    9. Set the running execution context’s LexicalEnvironment to oldEnv.
    10. Return R.

    13.12.9 Runtime Semantics: CaseBlockEvaluation

    CaseBlock : { CaseClauses }

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
             [...]
             3. Let foundInB be the result of performing Strict Equality
                Comparison input === clauseSelector.[[value]].
           ii. If foundInB is true, then
               [...]
    8. If foundInB is true, return NormalCompletion(V).
    9. Let R be the result of evaluating DefaultClause.
    10. If R.[[value]] is not empty, let V = R.[[value]].
    11. If R is an abrupt completion, return Completion(UpdateEmpty(R, V)).
    12. Repeat for each CaseClause C in B (NOTE this is another complete
        iteration of the second CaseClauses)
        a. Let R be the result of evaluating CaseClause C.
        b. If R.[[value]] is not empty, let V = R.[[value]].
        c. If R is an abrupt completion, return Completion(UpdateEmpty(R, V)).
    13. Return NormalCompletion(V).
---*/

assert.sameValue(
  eval('1; switch ("a") { default: case "b": }'),
  undefined,
  'empty StatementList (lone case)'
);
assert.sameValue(
  eval('2; switch ("a") { default: case "b": 3; }'),
  3,
  'non-empy StatementList (lone case)'
);
assert.sameValue(
  eval('4; switch ("a") { default: case "b": case "c": }'),
  undefined,
  'empty StatementList (following an empty case)'
);
assert.sameValue(
  eval('5; switch ("a") { default: case "b": case "c": 6; }'),
  6,
  'non-empty StatementList (following an empty case)'
);
assert.sameValue(
  eval('7; switch ("a") { default: case "b": 8; case "c": }'),
  8,
  'empty StatementList (following a non-empty case)'
);
assert.sameValue(
  eval('9; switch ("a") { default: case "b": 10; case "c": 11; }'),
  11,
  'non-empty StatementList (following a non-empty case)'
);
