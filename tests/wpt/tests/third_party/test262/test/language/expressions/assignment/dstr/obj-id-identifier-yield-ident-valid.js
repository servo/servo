// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-id-identifier-yield-ident-valid.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: yield is a valid IdentifierReference in an AssignmentProperty outside of strict mode and generator functions. (AssignmentExpression)
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
var yield;

var result;
var vals = { yield: 3 };

result = { yield } = vals;

assert.sameValue(yield, 3);

assert.sameValue(result, vals);
