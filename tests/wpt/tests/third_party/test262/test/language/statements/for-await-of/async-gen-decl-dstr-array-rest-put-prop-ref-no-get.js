// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/array-rest-put-prop-ref-no-get.case
// - src/dstr-assignment-for-await/default/async-gen-decl.template
/*---
description: If the DestructuringAssignmentTarget of an AssignmentElement is a PropertyReference, it should not be evaluated. (for-await-of statement in an async generator declaration)
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
let setValue;
let x = {
  get y() {
    throw new Test262Error('The property should not be accessed.');
  },
  set y(val) {
    setValue = val;
  }
};

let iterCount = 0;
async function * fn() {
  for await ([...x.y] of [[23, 45, 99]]) {
    assert.sameValue(setValue.length, 3);
    assert.sameValue(setValue[0], 23);
    assert.sameValue(setValue[1], 45);
    assert.sameValue(setValue[2], 99);
    iterCount += 1;
  }
}

let promise = fn().next();

promise
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);
