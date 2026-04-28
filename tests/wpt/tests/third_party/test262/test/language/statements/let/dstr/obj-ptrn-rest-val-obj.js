// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-val-obj.case
// - src/dstr-binding/default/let-stmt.template
/*---
description: Rest object contains just unextracted data (`let` statement)
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
features: [object-rest, destructuring-binding]
flags: [generated]
includes: [propertyHelper.js]
info: |
    LexicalBinding : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let value be GetValue(rhs).
    3. ReturnIfAbrupt(value).
    4. Let env be the running execution context's LexicalEnvironment.
    5. Return the result of performing BindingInitialization for BindingPattern
       using value and env as the arguments.
---*/

let {a, b, ...rest} = {x: 1, y: 2, a: 5, b: 3};

assert.sameValue(rest.a, undefined);
assert.sameValue(rest.b, undefined);

verifyProperty(rest, "x", {
  enumerable: true,
  writable: true,
  configurable: true,
  value: 1
});

verifyProperty(rest, "y", {
  enumerable: true,
  writable: true,
  configurable: true,
  value: 2
});
