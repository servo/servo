// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-iteration.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Value iteration only proceeds for the number of elements in the ArrayAssignmentPattern. (AssignmentExpression)
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

var result;
var vals = g();

result = [,,] = vals;

assert.sameValue(count, 2);

assert.sameValue(result, vals);
