// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-nested-obj.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: When DestructuringAssignmentTarget is an object literal, it should be parsed as a DestructuringAssignmentPattern and evaluated as a destructuring assignment. (AssignmentExpression)
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
var result, y;

var result;
var vals = { x: { y: 2 } };

result = { x: { y } } = vals;

assert.sameValue(y, 2);

assert.sameValue(result, vals);
