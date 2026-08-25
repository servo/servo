// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-target-identifier.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Identifiers that appear as the DestructuringAssignmentTarget in an AssignmentElement should take on the iterated value corresponding to their position in the ArrayAssignmentPattern. (AssignmentExpression)
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
var x, y, z;

var result;
var vals = [1, 2, 3];

result = [x, y, z] = vals;

assert.sameValue(x, 1);
assert.sameValue(y, 2);
assert.sameValue(z, 3);

assert.sameValue(result, vals);
