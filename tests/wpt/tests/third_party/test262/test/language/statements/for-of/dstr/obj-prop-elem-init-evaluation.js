// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-elem-init-evaluation.case
// - src/dstr-assignment/default/for-of.template
/*---
description: The Initializer should only be evaluated if v is undefined. (For..of statement)
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
var flag1 = false;
var flag2 = false;
var x, y;

var counter = 0;

for ({ x: x = flag1 = true, y: y = flag2 = true } of [{ y: 1 }]) {
  assert.sameValue(x, true, 'value of `x`');
  assert.sameValue(flag1, true, 'value of `flag1`');
  assert.sameValue(y, 1, 'value of `y`');
  assert.sameValue(flag2, false, 'value of `flag2`');
  counter += 1;
}

assert.sameValue(counter, 1);
