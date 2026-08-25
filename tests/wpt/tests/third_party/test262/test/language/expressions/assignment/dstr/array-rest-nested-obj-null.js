// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-nested-obj-null.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: When DestructuringAssignmentTarget is an object literal and the iterable emits `null` as the only value, an array with a single `null` element should be used as the value of the nested DestructuringAssignment. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var x, length;

var result;
var vals = [null];

result = [...{ 0: x, length }] = vals;

assert.sameValue(x, null);
assert.sameValue(length, 1);

assert.sameValue(result, vals);
