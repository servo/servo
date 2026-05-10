// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.12.11
description: Completion value when the default case matches and is final
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
       [...]
    8. If foundInB is true, return NormalCompletion(V).
    9. Let R be the result of evaluating DefaultClause.
    10. If R.[[value]] is not empty, let V = R.[[value]].
    [...]
    13. Return NormalCompletion(V).
---*/

assert.sameValue(
  eval('1; switch ("a") { default: }'),
  undefined,
  'empty StatementList (lone case)'
);
assert.sameValue(
  eval('2; switch ("a") { default: 3; }'),
  3,
  'non-empy StatementList (lone case)'
);
assert.sameValue(
  eval('4; switch ("b") { case "a": default: }'),
  undefined,
  'empty StatementList (following an empty case)'
);
assert.sameValue(
  eval('5; switch ("b") { case "a": default: 6; }'),
  6,
  'non-empty StatementList (following an empty case)'
);
assert.sameValue(
  eval('7; switch ("b") { case "a": 8; default: }'),
  undefined,
  'empty StatementList (following a non-empty case)'
);
assert.sameValue(
  eval('9; switch ("b") { case "a": 10; default: 11; }'),
  11,
  'non-empty StatementList (following a non-empty case)'
);
