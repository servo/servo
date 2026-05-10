// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-id-identifier-yield-ident-valid.case
// - src/dstr-assignment/default/for-of.template
/*---
description: yield is a valid IdentifierReference in an AssignmentProperty outside of strict mode and generator functions. (For..of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [destructuring-binding]
flags: [generated, noStrict]
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
var yield;

var counter = 0;

for ({ yield } of [{ yield: 3 }]) {
  assert.sameValue(yield, 3);
  counter += 1;
}

assert.sameValue(counter, 1);
