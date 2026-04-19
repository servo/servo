// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-init-evaluation.case
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
var flag1 = false, flag2 = false;
var _;

var result;
var vals = [14];

result = [ _ = flag1 = true, _ = flag2 = true ] = vals;

assert.sameValue(flag1, false);
assert.sameValue(flag2, true);

assert.sameValue(result, vals);
