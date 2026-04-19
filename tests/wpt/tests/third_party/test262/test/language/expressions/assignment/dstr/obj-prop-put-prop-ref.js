// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-put-prop-ref.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: The DestructuringAssignmentTarget of an AssignmentElement may be a PropertyReference. (AssignmentExpression)
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
var x = {};

var result;
var vals = { xy: 4 };

result = { xy: x.y } = vals;

assert.sameValue(x.y, 4);

assert.sameValue(result, vals);
