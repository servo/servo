// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-after-elision.case
// - src/dstr-assignment/default/for-of.template
/*---
description: An AssignmentRestElement following an elision consumes all remaining iterable values. (For..of statement)
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
var x;

var counter = 0;

for ([, ...x] of [[1, 2, 3]]) {
  assert.sameValue(x.length, 2);
  assert.sameValue(x[0], 2);
  assert.sameValue(x[1], 3);
  counter += 1;
}

assert.sameValue(counter, 1);
