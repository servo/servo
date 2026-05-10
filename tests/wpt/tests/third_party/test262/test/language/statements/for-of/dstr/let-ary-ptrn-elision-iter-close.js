// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elision-iter-close.case
// - src/dstr-binding/iter-close/for-of-let.template
/*---
description: The iterator is properly consumed by the destructuring pattern (for-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [generators, destructuring-binding]
flags: [generated]
info: |
    IterationStatement :
        for ( ForDeclaration of AssignmentExpression ) Statement

    [...]
    3. Return ForIn/OfBodyEvaluation(ForDeclaration, Statement, keyResult,
       lexicalBinding, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    3. Let destructuring be IsDestructuring of lhs.
    [...]
    5. Repeat
       [...]
       h. If destructuring is false, then
          [...]
       i. Else
          i. If lhsKind is assignment, then
             [...]
          ii. Else if lhsKind is varBinding, then
              [...]
          iii. Else,
               1. Assert: lhsKind is lexicalBinding.
               2. Assert: lhs is a ForDeclaration.
               3. Let status be the result of performing BindingInitialization
                  for lhs passing nextValue and iterationEnv as arguments.
          [...]
---*/
const iter = (function* () {
  yield;
  yield;
})();


function fn() {
  for (let [,] of [iter]) {
    return;
  }
}

fn();

assert.sameValue(iter.next().done, true, 'iteration occurred as expected');

