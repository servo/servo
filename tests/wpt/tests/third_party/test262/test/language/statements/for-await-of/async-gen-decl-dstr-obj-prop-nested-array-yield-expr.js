// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/obj-prop-nested-array-yield-expr.case
// - src/dstr-assignment-for-await/async-generator/async-gen-decl.template
/*---
description: When a `yield` token appears within the Initializer of a nested destructuring assignment and within a generator function body, it should behave as a YieldExpression. (for-await-of statement in an async generator declaration)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [generators, destructuring-binding, async-iteration]
flags: [generated, async]
info: |
    IterationStatement :
      for await ( LeftHandSideExpression of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ? ForIn/OfHeadEvaluation(« »,
       AssignmentExpression, iterate).
    2. Return ? ForIn/OfBodyEvaluation(LeftHandSideExpression, Statement,
       keyResult, assignment, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    5. If destructuring is true and if lhsKind is assignment, then
       a. Assert: lhs is a LeftHandSideExpression.
       b. Let assignmentPattern be the parse of the source text corresponding to
          lhs using AssignmentPattern as the goal symbol.
    [...]
---*/
let x;


let iterCount = 0;
async function * fn() {
  for await ({ x: [x = yield] } of [{ x: [] }]) {
    
    iterCount += 1;
  }
}

let iter = fn();

iter.next().then(result => {
  assert.sameValue(result.value, undefined);
  assert.sameValue(result.done, false);
  assert.sameValue(x, undefined);

  iter.next(24601).then(result => {
    assert.sameValue(result.value, undefined);
    assert.sameValue(result.done, true);
    assert.sameValue(x, 24601);
  }).then($DONE, $DONE);
});
