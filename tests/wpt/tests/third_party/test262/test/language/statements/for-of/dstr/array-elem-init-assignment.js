// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-init-assignment.case
// - src/dstr-assignment/default/for-of.template
/*---
description: If the Initializer is present and v is undefined, the Initializer should be evaluated and the result assigned to the target reference. (For..of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [destructuring-binding]
flags: [generated]
info: |
    IterationStatement :
      for ( LeftHandSideExpression of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ? ForIn/OfHeadEvaluation(« »,
       AssignmentExpression, iterate).
    2. Return ? ForIn/OfBodyEvaluation(LeftHandSideExpression, Statement,
       keyResult, assignment, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    4. If destructuring is true and if lhsKind is assignment, then
       a. Assert: lhs is a LeftHandSideExpression.
       b. Let assignmentPattern be the parse of the source text corresponding to
          lhs using AssignmentPattern as the goal symbol.
    [...]
---*/
var v2, vNull, vHole, vUndefined, vOob;

var counter = 0;

for ([v2 = 10, vNull = 11, vHole = 12, vUndefined = 13, vOob = 14] of [[2, null, , undefined]]) {
  assert.sameValue(v2, 2);
  assert.sameValue(vNull, null);
  assert.sameValue(vHole, 12);
  assert.sameValue(vUndefined, 13);
  assert.sameValue(vOob, 14);
  counter += 1;
}

assert.sameValue(counter, 1);
