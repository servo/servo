// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-symbol-val.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: RestBindingInitialization creates a new object if lhs is a Symbol (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [Symbol, object-rest, destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var rest;


var result;
var vals = Symbol("foo");

result = {...rest} = vals;

assert.notSameValue(rest, undefined);
assert.notSameValue(rest, null);
assert(rest instanceof Object);


assert.sameValue(result, vals);
