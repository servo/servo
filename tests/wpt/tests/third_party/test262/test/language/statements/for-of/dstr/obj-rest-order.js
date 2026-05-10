// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-order.case
// - src/dstr-assignment/default/for-of.template
/*---
description: Rest operation follows [[OwnPropertyKeys]] order (For..of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [Symbol, object-rest, destructuring-binding]
flags: [generated]
includes: [compareArray.js]
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
var rest;
var calls = [];
var o = { get z() { calls.push('z') }, get a() { calls.push('a') } };
Object.defineProperty(o, 1, { get: () => { calls.push(1) }, enumerable: true });
Object.defineProperty(o, Symbol('foo'), { get: () => { calls.push("Symbol(foo)") }, enumerable: true });

var counter = 0;

for ({...rest} of [o]) {
  assert.compareArray(calls, [1, 'z', 'a', "Symbol(foo)"]);
  assert.sameValue(Object.keys(rest).length, 3);
  counter += 1;
}

assert.sameValue(counter, 1);
