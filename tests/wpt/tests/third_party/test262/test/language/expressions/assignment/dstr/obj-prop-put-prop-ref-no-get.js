// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-put-prop-ref-no-get.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: If the DestructuringAssignmentTarget of an AssignmentElement is a PropertyReference, it should not be evaluated. (AssignmentExpression)
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
var setValue;
var x = {
  get y() {
    throw new Test262Error('The property should not be accessed.');
  },
  set y(val) {
    setValue = val;
  }
};

var result;
var vals = { a: 23 };

result = { a: x.y } = vals;

assert.sameValue(setValue, 23);

assert.sameValue(result, vals);
