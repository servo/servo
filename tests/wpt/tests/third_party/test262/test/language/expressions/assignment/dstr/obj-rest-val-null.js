// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-val-null.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: TypeError is thrown when rhs is null because of 7.1.13 ToObject ( argument ) used by CopyDataProperties (AssignmentExpression)
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

assert.throws(TypeError, function() {
  0, {...rest} = null
;
});
