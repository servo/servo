// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-identifier-resolution-trlng.case
// - src/dstr-assignment/default/for-of.template
/*---
description: Evaluation of DestructuringAssignmentTarget (lone element with trailing comma). (For..of statement)
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
var x = null;

var counter = 0;

for ({ a: x, } of [{ a: 2 }]) {
  assert.sameValue(x, 2);
  counter += 1;
}

assert.sameValue(counter, 1);
