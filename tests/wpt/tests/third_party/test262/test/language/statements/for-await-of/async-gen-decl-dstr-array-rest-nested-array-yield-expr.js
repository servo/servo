// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/array-rest-nested-array-yield-expr.case
// - src/dstr-assignment-for-await/async-generator/async-gen-decl.template
/*---
description: When a `yield` token appears within the DestructuringAssignmentTarget of a nested destructuring assignment and within a generator function body, it should behave as a YieldExpression. (for-await-of statement in an async generator declaration)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [generators, destructuring-binding, async-iteration]
flags: [generated, async]
info: |
    IterationStatement :
      for await ( LeftHandSideExpression of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ? ForIn/OfHeadEvaluation(« »,
       AssignmentExpression, iterate).
    2. Return ? ForIn/OfBodyEvaluation(LeftHandSideExpression, Statement,
       keyResult, assignment, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    5. If destructuring is true and if lhsKind is assignment, then
       a. Assert: lhs is a LeftHandSideExpression.
       b. Let assignmentPattern be the parse of the source text corresponding to
          lhs using AssignmentPattern as the goal symbol.
    [...]
---*/
let x = {};


let iterCount = 0;
async function * fn() {
  for await ([...[x[yield]]] of [[86]]) {
    
    iterCount += 1;
  }
}

let iter = fn();

iter.next().then(iterationResult => {
  assert.sameValue(iterationResult.value, undefined);
  assert.sameValue(iterationResult.done, false);
  assert.sameValue(x.prop, undefined);

  iter.next('prop').then(iterationResult => {
    assert.sameValue(iterationResult.value, undefined);
    assert.sameValue(iterationResult.done, true);
    assert.sameValue(x.prop, 86);
  }).then($DONE, $DONE);
});
