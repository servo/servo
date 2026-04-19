// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-non-string-computed-property-1dot0.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Destructuring field can be a non-string computed property, i.e it can be defined only at runtime. Rest operation needs to skip these properties as well. (AssignmentExpression)
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
var a = 1.;
var b, rest;

var result;
var vals = {[a]: 1.0, bar: 2 };

result = {[a]:b, ...rest} = vals;

assert.sameValue(b, 1);

assert.sameValue(Object.getOwnPropertyDescriptor(rest, "1"), undefined);

verifyProperty(rest, "bar", {
  value: 2,
  enumerable: true,
  writable: true,
  configurable: true
});


assert.sameValue(result, vals);
