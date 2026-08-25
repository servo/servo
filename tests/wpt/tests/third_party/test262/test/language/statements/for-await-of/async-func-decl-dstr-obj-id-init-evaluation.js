// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/obj-id-init-evaluation.case
// - src/dstr-assignment-for-await/default/async-func-decl.template
/*---
description: The Initializer should only be evaluated if v is undefined. (for-await-of statement in an async function declaration)
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
let flag1 = false;
let flag2 = false;
let x, y;

let iterCount = 0;
async function fn() {
  for await ({ x = flag1 = true, y = flag2 = true } of [{ y: 1 }]) {
    assert.sameValue(flag1, true);
    assert.sameValue(flag2, false);

    iterCount += 1;
  }
}

let promise = fn();

promise
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);
