// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-str-val.case
// - src/dstr-assignment/default/for-of.template
/*---
description: RestBindingInitialization creats an object with indexes as property name (For..of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [object-rest, destructuring-binding]
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
var rest;


var counter = 0;

for ({...rest} of ["foo"]) {
  assert.sameValue(rest["0"], "f");
  assert.sameValue(rest["1"], "o");
  assert.sameValue(rest["2"], "o");
  assert(rest instanceof Object);

  counter += 1;
}

assert.sameValue(counter, 1);
