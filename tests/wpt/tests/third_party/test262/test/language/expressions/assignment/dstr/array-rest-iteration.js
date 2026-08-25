// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-iteration.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: In the presense of an AssignmentRestElement, value iteration exhausts the iterable value; (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [generators, destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var count = 0;
var g = function*() {
  count += 1;
  yield;
  count += 1;
  yield;
  count += 1;
}
var x;

var result;
var vals = g();

result = [...x] = vals;

assert.sameValue(count, 3);

assert.sameValue(result, vals);
