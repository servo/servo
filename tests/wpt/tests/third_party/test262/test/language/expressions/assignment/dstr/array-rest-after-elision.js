// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-after-elision.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: An AssignmentRestElement following an elision consumes all remaining iterable values. (AssignmentExpression)
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
var vals = [1, 2, 3];

result = [, ...x] = vals;

assert.sameValue(x.length, 2);
assert.sameValue(x[0], 2);
assert.sameValue(x[1], 3);

assert.sameValue(result, vals);
