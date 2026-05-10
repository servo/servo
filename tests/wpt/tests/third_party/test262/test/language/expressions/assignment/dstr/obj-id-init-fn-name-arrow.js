// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-id-init-fn-name-arrow.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Assignment of function `name` attribute (ArrowFunction) (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated]
includes: [propertyHelper.js]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.

    AssignmentProperty : IdentifierReference Initializeropt
    [...] 6. If Initializeropt is present and v is undefined, then
       [...]
       d. If IsAnonymousFunctionDefinition(Initializer) is true, then
          i. Let hasNameProperty be HasOwnProperty(v, "name").
          ii. ReturnIfAbrupt(hasNameProperty).
          iii. If hasNameProperty is false, perform SetFunctionName(v, P).

---*/
var arrow;

var result;
var vals = {};

result = { arrow = () => {} } = vals;

verifyProperty(arrow, 'name', {
  enumerable: false,
  writable: false,
  configurable: true,
  value: 'arrow'
});

assert.sameValue(result, vals);
