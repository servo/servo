// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/obj-rest-skip-non-enumerable.case
// - src/dstr-assignment-for-await/default/async-func-decl.template
/*---
description: Rest object doesn't contain non-enumerable properties (for-await-of statement in an async function declaration)
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
let rest;
let obj = {a: 3, b: 4};
Object.defineProperty(obj, "x", { value: 4, enumerable: false });

let iterCount = 0;
async function fn() {
  for await ({...rest} of [obj]) {
    assert.sameValue(Object.getOwnPropertyDescriptor(rest, "x"), undefined);

    verifyProperty(rest, "a", {
      enumerable: true,
      writable: true,
      configurable: true,
      value: 3
    });

    verifyProperty(rest, "b", {
      enumerable: true,
      writable: true,
      configurable: true,
      value: 4
    });

    iterCount += 1;
  }
}

let promise = fn();

promise
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);
