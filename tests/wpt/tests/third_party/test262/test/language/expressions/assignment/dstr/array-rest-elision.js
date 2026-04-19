// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-elision.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: ArrayAssignmentPattern may include elisions at any position preceding a AssignmentRestElement in a AssignmentElementList. (AssignmentExpression)
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
var x, y;

var result;
var vals = [1, 2, 3, 4, 5, 6];

result = [, , x, , ...y] = vals;

assert.sameValue(x, 3);
assert.sameValue(y.length, 2);
assert.sameValue(y[0], 5);
assert.sameValue(y[1], 6);

assert.sameValue(result, vals);
