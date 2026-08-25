// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-init-assignment.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: If the Initializer is present and v is undefined, the Initializer should be evaluated and the result assigned to the target reference. (AssignmentExpression)
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
var v2, vNull, vHole, vUndefined, vOob;

var result;
var vals = [2, null, , undefined];

result = [v2 = 10, vNull = 11, vHole = 12, vUndefined = 13, vOob = 14] = vals;

assert.sameValue(v2, 2);
assert.sameValue(vNull, null);
assert.sameValue(vHole, 12);
assert.sameValue(vUndefined, 13);
assert.sameValue(vOob, 14);

assert.sameValue(result, vals);
