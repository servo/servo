// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-elem-init-let.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: Value retrieval of Initializer obeys `let` semantics. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [let, destructuring-binding]
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

assert.throws(ReferenceError, function() {
  0, { x: x = y } = {};
});

let y;
