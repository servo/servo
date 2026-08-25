// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/array-elem-nested-array-null.case
// - src/dstr-assignment-for-await/default/async-gen-decl.template
/*---
description: When DestructuringAssignmentTarget is an array literal and the value is `null`, a TypeError should be thrown. (for-await-of statement in an async generator declaration)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [destructuring-binding, async-iteration]
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
let _;


let iterCount = 0;
async function * fn() {
  for await ([[ _ ]] of [[null]]) {
    
    iterCount += 1;
  }
}

let promise = fn().next();

promise.then(() => $DONE('Promise incorrectly fulfilled.'), ({ constructor }) => {
  assert.sameValue(iterCount, 0);
  assert.sameValue(constructor, TypeError);
}).then($DONE, $DONE);

