// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-skip-non-enumerable.case
// - src/dstr-binding/default/let-stmt.template
/*---
description: Rest object doesn't contain non-enumerable properties (`let` statement)
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
var o = {a: 3, b: 4};
Object.defineProperty(o, "x", { value: 4, enumerable: false });

let {...rest} = o;

assert.sameValue(rest.x, undefined);

verifyProperty(rest, "a", {
  enumerable: true,
  writable: true,
  configurable: true,
  value: 3
});

verifyProperty(rest, "b", {
  enumerable: true,
  writable: true,
  configurable: true,
  value: 4
});
