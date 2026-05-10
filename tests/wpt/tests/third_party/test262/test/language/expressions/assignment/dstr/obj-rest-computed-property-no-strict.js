// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-rest-computed-property-no-strict.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Destructuring field can be a computed property, i.e it can be defined only at runtime. Rest operantion needs to skip these properties as well. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [object-rest, destructuring-binding]
flags: [generated, noStrict]
includes: [propertyHelper.js]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.
---*/
var a = "foo";

var result;
var vals = { foo: 1, bar: 2, baz: 3 };

result = {[a]:b, ...rest} = vals;

assert.sameValue(b, 1);
assert.sameValue(rest.bar, 2);
assert.sameValue(rest.baz, 3);

assert.sameValue(Object.getOwnPropertyDescriptor(rest, "foo"), undefined);

verifyProperty(rest, "bar", {
  enumerable: true,
  writable: true,
  configurable: true
});

verifyProperty(rest, "baz", {
  enumerable: true,
  writable: true,
  configurable: true
});


assert.sameValue(result, vals);
