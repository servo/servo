// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-id-simple-no-strict.case
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
var vals = { eval: 1, arguments: 2 };

result = { eval, arguments } = vals;



assert.sameValue(result, vals);

assert.sameValue(eval, 1);
assert.sameValue(arguments, 2);
