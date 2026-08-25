// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-init-yield-expr.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: When a `yield` token appears within the Initializer of an AssignmentElement within a generator function body, it behaves as a YieldExpression. (AssignmentExpression)
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
var value = [];
var iterationResult, iter, x;
iter = (function*() {

var result;
var vals = [];

result = [ x = yield ] = vals;



assert.sameValue(result, vals);

})();

iterationResult = iter.next();

assert.sameValue(iterationResult.value, undefined);
assert.sameValue(iterationResult.done, false);
assert.sameValue(x, undefined);

iterationResult = iter.next(86);

assert.sameValue(iterationResult.value, undefined);
assert.sameValue(iterationResult.done, true);
assert.sameValue(x, 86);
