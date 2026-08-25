// This file was procedurally generated from the following sources:
// - src/spread/mult-obj-ident.case
// - src/spread/default/member-expr.template
/*---
description: Object Spread operator following other properties (`new` operator)
esid: sec-new-operator-runtime-semantics-evaluation
features: [object-spread]
flags: [generated]
includes: [propertyHelper.js]
info: |
    MemberExpression : new MemberExpression Arguments

    1. Return EvaluateNew(MemberExpression, Arguments).

    12.3.3.1.1 Runtime Semantics: EvaluateNew

    6. If arguments is empty, let argList be an empty List.
    7. Else,
       a. Let argList be ArgumentListEvaluation of arguments.
       [...]

    Pending Runtime Semantics: PropertyDefinitionEvaluation

    PropertyDefinition:...AssignmentExpression

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let fromValue be GetValue(exprValue).
    3. ReturnIfAbrupt(fromValue).
    4. Let excludedNames be a new empty List.
    5. Return CopyDataProperties(object, fromValue, excludedNames).

---*/
let o = {c: 3, d: 4};


var callCount = 0;

new function(obj) {
  assert.sameValue(Object.keys(obj).length, 4);

  verifyProperty(obj, "a", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 1
  });

  verifyProperty(obj, "b", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 2
  });

  verifyProperty(obj, "c", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 3
  });

  verifyProperty(obj, "d", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 4
  });
  callCount += 1;
}({a: 1, b: 2, ...o});

assert.sameValue(callCount, 1);
