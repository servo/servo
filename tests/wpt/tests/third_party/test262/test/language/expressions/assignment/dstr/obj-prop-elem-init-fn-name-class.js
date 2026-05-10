// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-elem-init-fn-name-class.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Assignment of function `name` attribute (ClassExpression) (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [class, destructuring-binding]
flags: [generated]
includes: [propertyHelper.js]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.

    AssignmentElement[Yield] : DestructuringAssignmentTarget Initializeropt
    [...] 7. If Initializer is present and v is undefined and
       IsAnonymousFunctionDefinition(Initializer) and IsIdentifierRef of
       DestructuringAssignmentTarget are both true, then
       a. Let hasNameProperty be HasOwnProperty(rhsValue, "name").
       b. ReturnIfAbrupt(hasNameProperty).
       c. If hasNameProperty is false, perform SetFunctionName(rhsValue,
          GetReferencedName(lref)).

---*/
var xCls, cls, xCls2;

var result;
var vals = {};

result = { x: xCls = class x {}, x: cls = class {}, x: xCls2 = class { static name() {} } } = vals;

assert.notSameValue(xCls.name, 'xCls');
assert.notSameValue(xCls2.name, 'xCls2');

verifyProperty(cls, 'name', {
  enumerable: false,
  writable: false,
  configurable: true,
  value: 'cls'
});

assert.sameValue(result, vals);
