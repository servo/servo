// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-name-evaluation-error.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: Any error raised as a result of evaluating PropertyName should be forwarded to the runtime. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var a, x;

assert.throws(TypeError, function() {
  0, { [a.b]: x } = {};
});
