// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-elem-target-yield-ident-valid.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: When a `yield` token appears within the DestructuringAssignmentTarget of an AssignmentElement and outside of a generator function body, it should behave as an IdentifierReference. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated, noStrict]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var yield = 'prop';
var x = {};

var result;
var vals = { x: 23 };

result = { x: x[yield] } = vals;

assert.sameValue(x.prop, 23);

assert.sameValue(result, vals);
