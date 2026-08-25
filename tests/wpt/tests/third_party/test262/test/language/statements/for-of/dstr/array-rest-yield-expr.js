// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-yield-expr.case
// - src/dstr-assignment/default/for-of.template
/*---
description: When a `yield` token appears within the DestructuringAssignmentTarget of an AssignmentRestElement and within the body of a generator function, it should behave as a YieldExpression. (For..of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [generators, destructuring-binding]
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
var x = {};
var iterationResult, iter;

iter = (function*() {

var counter = 0;

for ([...x[yield]] of [[33, 44, 55]]) {
  
  counter += 1;
}

assert.sameValue(counter, 1);

}());

iterationResult = iter.next();

assert.sameValue(iterationResult.value, undefined);
assert.sameValue(iterationResult.done, false);
assert.sameValue(x.prop, undefined);

iterationResult = iter.next('prop');

assert.sameValue(iterationResult.value, undefined);
assert.sameValue(iterationResult.done, true);
assert.sameValue(x.prop.length, 3);
assert.sameValue(x.prop[0], 33);
assert.sameValue(x.prop[1], 44);
assert.sameValue(x.prop[2], 55);
