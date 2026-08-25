// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-to-property.case
// - src/dstr-assignment/default/for-of.template
/*---
description: When DestructuringAssignmentTarget is an object property, its value should be binded as rest object. (For..of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [object-rest, destructuring-binding]
flags: [generated]
includes: [propertyHelper.js]
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
var src = {};

var counter = 0;

for ({...src.y} of [{ x: 1, y: 2}]) {
  assert.sameValue(src.y.x, 1);
  assert.sameValue(src.y.y, 2);

  verifyProperty(src, "y", {
    enumerable: true,
    writable: true,
    configurable: true
  });
  counter += 1;
}

assert.sameValue(counter, 1);
