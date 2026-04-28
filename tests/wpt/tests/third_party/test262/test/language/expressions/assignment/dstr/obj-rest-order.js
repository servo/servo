// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-order.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Rest operation follows [[OwnPropertyKeys]] order (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [Symbol, object-rest, destructuring-binding]
flags: [generated]
includes: [compareArray.js]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var rest;
var calls = [];
var o = { get z() { calls.push('z') }, get a() { calls.push('a') } };
Object.defineProperty(o, 1, { get: () => { calls.push(1) }, enumerable: true });
Object.defineProperty(o, Symbol('foo'), { get: () => { calls.push("Symbol(foo)") }, enumerable: true });

var result;
var vals = o;

result = {...rest} = vals;

assert.compareArray(calls, [1, 'z', 'a', "Symbol(foo)"]);
assert.sameValue(Object.keys(rest).length, 3);

assert.sameValue(result, vals);
