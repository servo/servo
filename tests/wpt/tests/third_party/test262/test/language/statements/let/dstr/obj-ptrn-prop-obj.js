// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-obj.case
// - src/dstr-binding/default/let-stmt.template
/*---
description: Object binding pattern with "nested" object binding pattern not using initializer (`let` statement)
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated]
info: |
    LexicalBinding : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let value be GetValue(rhs).
    3. ReturnIfAbrupt(value).
    4. Let env be the running execution context's LexicalEnvironment.
    5. Return the result of performing BindingInitialization for BindingPattern
       using value and env as the arguments.

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    [...]
    3. If Initializer is present and v is undefined, then
       [...]
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/

let { w: { x, y, z } = { x: 4, y: 5, z: 6 } } = { w: { x: undefined, z: 7 } };

assert.sameValue(x, undefined);
assert.sameValue(y, undefined);
assert.sameValue(z, 7);

assert.throws(ReferenceError, function() {
  w;
});
