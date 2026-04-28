// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-put-unresolvable-no-strict.case
// - src/dstr-assignment/default/for-of.template
/*---
description: Outside of strict mode, if the the assignment target is an unresolvable reference, a new `var` binding should be created in the environment record. (For..of statement)
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
{

var counter = 0;

for ([ ...unresolvable ] of [[]]) {
  
  counter += 1;
}

assert.sameValue(counter, 1);

}

assert.sameValue(unresolvable.length, 0);
