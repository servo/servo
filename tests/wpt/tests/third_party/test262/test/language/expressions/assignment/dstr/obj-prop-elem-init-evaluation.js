// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-elem-init-evaluation.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: The Initializer should only be evaluated if v is undefined. (AssignmentExpression)
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
var flag1 = false;
var flag2 = false;
var x, y;

var result;
var vals = { y: 1 };

result = { x: x = flag1 = true, y: y = flag2 = true } = vals;

assert.sameValue(x, true, 'value of `x`');
assert.sameValue(flag1, true, 'value of `flag1`');
assert.sameValue(y, 1, 'value of `y`');
assert.sameValue(flag2, false, 'value of `flag2`');

assert.sameValue(result, vals);
