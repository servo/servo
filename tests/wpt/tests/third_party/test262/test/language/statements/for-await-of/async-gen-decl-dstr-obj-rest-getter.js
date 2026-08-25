// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/obj-rest-getter.case
// - src/dstr-assignment-for-await/default/async-gen-decl.template
/*---
description: Getter is called when obj is being deconstructed to a rest Object (for-await-of statement in an async generator declaration)
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
let x;
let count = 0;

let iterCount = 0;
async function * fn() {
  for await ({...x} of [{ get v() { count++; return 2; } }]) {
    assert.sameValue(count, 1);

    verifyProperty(x, "v", {
      enumerable: true,
      writable: true,
      configurable: true,
      value: 2
    });

    iterCount += 1;
  }
}

let promise = fn().next();

promise
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);
