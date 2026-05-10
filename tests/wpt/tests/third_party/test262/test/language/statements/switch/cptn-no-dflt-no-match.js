// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.12.11
description: Completion value when no cases match
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
    2. Let A be the List of CaseClause items in CaseClauses, in source text
       order.
    3. Let found be false.
    4. Repeat for each CaseClause C in A,
       a. If found is false, then
          i. Let clauseSelector be the result of CaseSelectorEvaluation of C.
          ii. If clauseSelector is an abrupt completion, then
              [...]
          iii. Let found be the result of performing Strict Equality Comparison
               input === clauseSelector.[[value]].
       b. If found is true, then
          [...]
    5. Return NormalCompletion(V).
---*/

assert.sameValue(
  eval('1; switch ("a") { case null: }'), undefined, 'empty StatementList'
);
assert.sameValue(
  eval('2; switch ("a") { case null: 3; }'),
  undefined,
  'non-empty StatementList'
);
