// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-put-prop-ref-no-get.case
// - src/dstr-assignment/default/for-of.template
/*---
description: If the DestructuringAssignmentTarget of an AssignmentElement is a PropertyReference, it should not be evaluated. (For..of statement)
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
var x, setValue;
x = {
  get y() {
    throw new Test262Error('The property should not be accessed.');
  },
  set y(val) {
    setValue = val;
  }
};

var counter = 0;

for ([x.y] of [[23]]) {
  assert.sameValue(setValue, 23);
  counter += 1;
}

assert.sameValue(counter, 1);
