// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-yield-expr.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: When a `yield` token appears within the DestructuringAssignmentTarget of an AssignmentRestElement and within the body of a generator function, it should behave as a YieldExpression. (AssignmentExpression)
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
var x = {};
var iterationResult, iter;

iter = (function*() {

var result;
var vals = [33, 44, 55];

result = [...x[yield]] = vals;



assert.sameValue(result, vals);

}());

iterationResult = iter.next();

assert.sameValue(iterationResult.value, undefined);
assert.sameValue(iterationResult.done, false);
assert.sameValue(x.prop, undefined);

iterationResult = iter.next('prop');

assert.sameValue(iterationResult.value, undefined);
assert.sameValue(iterationResult.done, true);
assert.sameValue(x.prop.length, 3);
assert.sameValue(x.prop[0], 33);
assert.sameValue(x.prop[1], 44);
assert.sameValue(x.prop[2], 55);
