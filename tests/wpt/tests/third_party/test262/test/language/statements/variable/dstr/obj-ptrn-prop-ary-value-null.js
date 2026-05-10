// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-ary-value-null.case
// - src/dstr-binding/error/var-stmt.template
/*---
description: Object binding pattern with "nested" array binding pattern taking the `null` value (`var` statement)
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

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    [...]
    3. If Initializer is present and v is undefined, then
       [...]
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/

assert.throws(TypeError, function() {
  var { w: [x, y, z] = [4, 5, 6] } = { w: null };
});
