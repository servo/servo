// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-computed-property-no-strict.case
// - src/dstr-assignment/default/for-of.template
/*---
description: Destructuring field can be a computed property, i.e it can be defined only at runtime. Rest operantion needs to skip these properties as well. (For..of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [object-rest, destructuring-binding]
flags: [generated, noStrict]
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
var a = "foo";

var counter = 0;

for ({[a]:b, ...rest} of [{ foo: 1, bar: 2, baz: 3 }]) {
  assert.sameValue(b, 1);
  assert.sameValue(rest.bar, 2);
  assert.sameValue(rest.baz, 3);

  assert.sameValue(Object.getOwnPropertyDescriptor(rest, "foo"), undefined);

  verifyProperty(rest, "bar", {
    enumerable: true,
    writable: true,
    configurable: true
  });

  verifyProperty(rest, "baz", {
    enumerable: true,
    writable: true,
    configurable: true
  });

  counter += 1;
}

assert.sameValue(counter, 1);
