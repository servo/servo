// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-obj-init.case
// - src/dstr-binding/default/var-stmt.template
/*---
description: Object binding pattern with "nested" object binding pattern using initializer (`var` statement)
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
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be GetValue(defaultValue).
       c. ReturnIfAbrupt(v).
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/

var { w: { x, y, z } = { x: 4, y: 5, z: 6 } } = { w: undefined };

assert.sameValue(x, 4);
assert.sameValue(y, 5);
assert.sameValue(z, 6);

assert.throws(ReferenceError, function() {
  w;
});
