// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-same-name.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Proper setting in the values for rest name equal to a property name. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [object-rest, destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var o = {
    x: 42,
    y: 39,
    z: 'cheeseburger'
};

var x, y, z;

var result;
var vals = o;

result = { x, ...z } = vals;

assert.sameValue(x, 42);
assert.sameValue(y, undefined);
assert.sameValue(z.y, 39);
assert.sameValue(z.z, 'cheeseburger');

var keys = Object.getOwnPropertyNames(z);
assert.sameValue(keys.length, 2);
assert.sameValue(keys[0], 'y');
assert.sameValue(keys[1], 'z');

assert.sameValue(result, vals);
