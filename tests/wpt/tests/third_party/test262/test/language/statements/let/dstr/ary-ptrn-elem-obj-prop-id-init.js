// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-obj-prop-id-init.case
// - src/dstr-binding/default/let-stmt.template
/*---
description: BindingElement with object binding pattern and initializer is used (`let` statement)
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

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    BindingElement : BindingPatternInitializer opt

    [...]
    2. If iteratorRecord.[[done]] is true, let v be undefined.
    3. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be ? GetValue(defaultValue).
    4. Return the result of performing BindingInitialization of BindingPattern
       with v and environment as the arguments.
---*/

let [{ u: v, w: x, y: z } = { u: 444, w: 555, y: 666 }] = [];

assert.sameValue(v, 444);
assert.sameValue(x, 555);
assert.sameValue(z, 666);

assert.throws(ReferenceError, function() {
  u;
});
assert.throws(ReferenceError, function() {
  w;
});
assert.throws(ReferenceError, function() {
  y;
});
