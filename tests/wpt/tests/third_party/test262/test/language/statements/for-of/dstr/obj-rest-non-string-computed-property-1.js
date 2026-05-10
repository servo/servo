// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-non-string-computed-property-1.case
// - src/dstr-assignment/default/for-of.template
/*---
description: Destructuring field can be a non-string computed property, i.e it can be defined only at runtime. Rest operation needs to skip these properties as well. (For..of statement)
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
var a = 1.;
var b, rest;

var counter = 0;

for ({[a]:b, ...rest} of [{[a]: 1, bar: 2 }]) {
  assert.sameValue(b, 1);

  assert.sameValue(Object.getOwnPropertyDescriptor(rest, "1"), undefined);

  verifyProperty(rest, "bar", {
    value: 2,
    enumerable: true,
    writable: true,
    configurable: true
  });

  counter += 1;
}

assert.sameValue(counter, 1);
