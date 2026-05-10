// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.12.11
description: >
    Completion value when the matching case is exited via an empty abrupt
    completion
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
              1. If clauseSelector.[[value]] is empty, return
                 Completion{[[type]]: clauseSelector.[[type]], [[value]]:
                 undefined, [[target]]: clauseSelector.[[target]]}.
              2. Else, return Completion(clauseSelector).
          iii. Let found be the result of performing Strict Equality Comparison
               input === clauseSelector.[[value]].
        b. If found is true, then
           i. Let R be the result of evaluating C.
           ii. If R.[[value]] is not empty, let V = R.[[value]].
           iii. If R is an abrupt completion, return Completion(UpdateEmpty(R,
                V)).
---*/

assert.sameValue(eval('1; switch ("a") { case "a": break; }'), undefined);
assert.sameValue(eval('2; switch ("a") { case "a": { 3; break; } }'), 3);

assert.sameValue(
  eval('4; do { switch ("a") { case "a": continue; } } while (false)'),
  undefined
);
assert.sameValue(
  eval('5; do { switch ("a") { case "a": { 6; continue; } } } while (false)'),
  6
);
