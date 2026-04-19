// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-descriptors.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Object created from rest deconstruction doesn't copy source object property descriptors. (AssignmentExpression)
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
var rest;
var obj = {};
Object.defineProperty(obj, "a", { value: 3, configurable: false, enumerable: true });
Object.defineProperty(obj, "b", { value: 4, writable: false, enumerable: true });

var result;
var vals = obj;

result = {...rest} = vals;

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


assert.sameValue(result, vals);
