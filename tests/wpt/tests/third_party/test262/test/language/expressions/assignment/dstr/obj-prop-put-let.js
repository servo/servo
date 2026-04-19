// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-put-let.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: The assignment target should obey `let` semantics. (AssignmentExpression)
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

assert.throws(ReferenceError, function() {
  0, { a: x } = {};
});

let x;
