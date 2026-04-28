// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-getter-abrupt-get-error.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: Rest deconstruction doesn't happen if getter return is abrupt (AssignmentExpression)
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
var x;
var count = 0;

assert.throws(Test262Error, function() {
  0, {...x} = { get v() { count++; throw new Test262Error(); } };
});

assert.sameValue(count, 1);

