// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-id-init-assignment-undef.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: If the Initializer is present and v is undefined, the Initializer should be evaluated and the result assigned to the target reference ("undefined" property value defined). (AssignmentExpression)
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
var x;

var result;
var vals = { x: undefined };

result = { x = 1 } = vals;

assert.sameValue(x, 1);

assert.sameValue(result, vals);
