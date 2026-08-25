// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-id-init-yield-expr.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: When a `yield` token appears within the Initializer of an AssignmentProperty and within a generator function body, it should behave as a YieldExpression. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [generators, destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var iterationResult, x, iter;

iter = (function*() {

var result;
var vals = {};

result = { x = yield } = vals;



assert.sameValue(result, vals);

}());

iterationResult = iter.next();

assert.sameValue(iterationResult.value, undefined);
assert.sameValue(iterationResult.done, false);
assert.sameValue(x, undefined);

iterationResult = iter.next(3);

assert.sameValue(iterationResult.value, undefined);
assert.sameValue(iterationResult.done, true);
assert.sameValue(x, 3);
