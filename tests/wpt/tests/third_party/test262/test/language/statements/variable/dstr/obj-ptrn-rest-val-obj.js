// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-val-obj.case
// - src/dstr-binding/default/var-stmt.template
/*---
description: Rest object contains just unextracted data (`var` statement)
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

var {a, b, ...rest} = {x: 1, y: 2, a: 5, b: 3};

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
