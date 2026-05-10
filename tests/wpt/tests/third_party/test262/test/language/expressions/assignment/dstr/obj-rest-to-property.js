// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-to-property.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: When DestructuringAssignmentTarget is an object property, its value should be binded as rest object. (AssignmentExpression)
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
var src = {};

var result;
var vals = { x: 1, y: 2};

result = {...src.y} = vals;

assert.sameValue(src.y.x, 1);
assert.sameValue(src.y.y, 2);

verifyProperty(src, "y", {
  enumerable: true,
  writable: true,
  configurable: true
});

assert.sameValue(result, vals);
