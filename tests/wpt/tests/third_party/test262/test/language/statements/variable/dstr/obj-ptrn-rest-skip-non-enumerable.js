// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-skip-non-enumerable.case
// - src/dstr-binding/default/var-stmt.template
/*---
description: Rest object doesn't contain non-enumerable properties (`var` statement)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [object-rest, destructuring-binding]
flags: [generated]
includes: [propertyHelper.js]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var o = {a: 3, b: 4};
Object.defineProperty(o, "x", { value: 4, enumerable: false });

var {...rest} = o;

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
