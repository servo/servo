// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/obj-rest-to-property.case
// - src/dstr-assignment-for-await/default/async-gen-decl.template
/*---
description: When DestructuringAssignmentTarget is an object property, its value should be binded as rest object. (for-await-of statement in an async generator declaration)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [object-rest, destructuring-binding, async-iteration]
flags: [generated, async]
includes: [propertyHelper.js]
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
let src = {};

let iterCount = 0;
async function * fn() {
  for await ({...src.y} of [{ x: 1, y: 2}]) {
    assert.sameValue(src.y.x, 1);
    assert.sameValue(src.y.y, 2);

    verifyProperty(src, "y", {
      enumerable: true,
      writable: true,
      configurable: true
    });

    iterCount += 1;
  }
}

let promise = fn().next();

promise
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);
