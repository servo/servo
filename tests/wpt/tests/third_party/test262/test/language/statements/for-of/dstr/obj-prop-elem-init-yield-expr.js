// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-elem-init-yield-expr.case
// - src/dstr-assignment/default/for-of.template
/*---
description: When a `yield` token appears within the Initializer of an AssignmentElement and within a generator function body, it should behave as a YieldExpression. (For..of statement)
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
var iterationResult, iter, x;
iter = (function*() {

var counter = 0;

for ({ x: x = yield } of [{}]) {
  
  counter += 1;
}

assert.sameValue(counter, 1);

}());

iterationResult = iter.next();

assert.sameValue(iterationResult.value, undefined);
assert.sameValue(iterationResult.done, false);
assert.sameValue(x, undefined);

iterationResult = iter.next(86);

assert.sameValue(iterationResult.value, undefined);
assert.sameValue(iterationResult.done, true);
assert.sameValue(x, 86);
