// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-getter.case
// - src/dstr-binding/default/let-stmt.template
/*---
description: Getter is called when obj is being deconstructed to a rest Object (`let` statement)
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
var count = 0;

let {...x} = { get v() { count++; return 2; } };

assert.sameValue(count, 1);

verifyProperty(x, "v", {
  enumerable: true,
  writable: true,
  configurable: true,
  value: 2
});
