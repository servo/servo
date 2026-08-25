// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-id-init-simple-no-strict.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Evaluation of DestructuringAssignmentTarget. (AssignmentExpression)
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
var eval, arguments;

var result;
var vals = {};

result = { eval = 3, arguments = 4 } = vals;

assert.sameValue(eval, 3);
assert.sameValue(arguments, 4);

assert.sameValue(result, vals);
