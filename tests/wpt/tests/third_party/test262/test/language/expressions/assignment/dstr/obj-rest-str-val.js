// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-str-val.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: RestBindingInitialization creats an object with indexes as property name (AssignmentExpression)
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
var rest;


var result;
var vals = "foo";

result = {...rest} = vals;

assert.sameValue(rest["0"], "f");
assert.sameValue(rest["1"], "o");
assert.sameValue(rest["2"], "o");
assert(rest instanceof Object);


assert.sameValue(result, vals);
