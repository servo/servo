// This file was procedurally generated from the following sources:
// - src/spread/sngl-obj-ident.case
// - src/spread/default/call-expr.template
/*---
description: Object Spread operator without other arguments (CallExpression)
esid: sec-function-calls-runtime-semantics-evaluation
features: [object-spread]
flags: [generated]
includes: [propertyHelper.js]
info: |
    CallExpression : MemberExpression Arguments

    [...]
    9. Return EvaluateDirectCall(func, thisValue, Arguments, tailCall).

    12.3.4.3 Runtime Semantics: EvaluateDirectCall

    1. Let argList be ArgumentListEvaluation(arguments).
    [...]
    6. Let result be Call(func, thisValue, argList).
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

(function(obj) {
  assert.sameValue(Object.keys(obj).length, 2);

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
}({...o}));

assert.sameValue(callCount, 1);
