// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-init-fn-name-gen.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Assignment of function `name` attribute (GeneratorExpression) (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [generators, destructuring-binding]
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
    [...] 7. If Initializer is present and value is undefined and
       IsAnonymousFunctionDefinition(Initializer) and IsIdentifierRef of
       DestructuringAssignmentTarget are both true, then
       a. Let hasNameProperty be HasOwnProperty(v, "name").
       b. ReturnIfAbrupt(hasNameProperty).
       c. If hasNameProperty is false, perform SetFunctionName(v,
          GetReferencedName(lref)).

---*/
var xGen, gen;

var result;
var vals = [];

result = [ xGen = function* x() {}, gen = function*() {} ] = vals;

assert.notSameValue(xGen.name, 'xGen');

verifyProperty(gen, 'name', {
  enumerable: false,
  writable: false,
  configurable: true,
  value: 'gen'
});

assert.sameValue(result, vals);
