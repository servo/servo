// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-getter.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Getter is called when obj is being deconstructed to a rest Object (AssignmentExpression)
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
var x;
var count = 0;

var result;
var vals = { get v() { count++; return 2; } };

result = {...x} = vals;

assert.sameValue(count, 1);

verifyProperty(x, "v", {
  enumerable: true,
  writable: true,
  configurable: true,
  value: 2
});

assert.sameValue(result, vals);
