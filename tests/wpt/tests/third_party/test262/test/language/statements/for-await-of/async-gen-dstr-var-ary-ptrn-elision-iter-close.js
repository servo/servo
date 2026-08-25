// This file was procedurally generated from the following sources:
// - src/dstr-binding-for-await/ary-ptrn-elision-iter-close.case
// - src/dstr-binding-for-await/iter-close/for-await-of-async-gen-var.template
/*---
description: The iterator is properly consumed by the destructuring pattern (for-await-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [generators, destructuring-binding, async-iteration]
flags: [generated, async]
info: |
    IterationStatement :
        for await ( var ForBinding of AssignmentExpression ) Statement

    [...]
    2. Return ? ForIn/OfBodyEvaluation(ForBinding, Statement, keyResult,
        varBinding, labelSet, async).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    4. Let destructuring be IsDestructuring of lhs.
    [...]
    6. Repeat
       [...]
       j. If destructuring is false, then
          [...]
       k. Else
          i. If lhsKind is assignment, then
             [...]
          ii. Else if lhsKind is varBinding, then
              1. Assert: lhs is a ForBinding.
              2. Let status be the result of performing BindingInitialization
                 for lhs passing nextValue and undefined as the arguments.
          [...]
---*/
const iter = (function* () {
  yield;
  yield;
})();


async function* fn() {
  for await (var [,] of [iter]) {
    return;
  }
}

fn()
  .next()
  .then(() => {
    assert.sameValue(iter.next().done, true, 'iteration occurred as expected');
  })
  .then($DONE, $DONE);
