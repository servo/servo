// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-valid-object.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Rest object contains just unextracted data (AssignmentExpression)
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
var rest, a, b;


var result;
var vals = {x: 1, y: 2, a: 5, b: 3};

result = {a, b, ...rest} = vals;

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

assert.sameValue(result, vals);
