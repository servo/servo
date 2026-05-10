// This file was procedurally generated from the following sources:
// - src/spread/mult-obj-undefined.case
// - src/spread/default/call-expr.template
/*---
description: Object Spread operator following other arguments with undefined (CallExpression)
esid: sec-function-calls-runtime-semantics-evaluation
features: [object-spread]
flags: [generated]
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

var callCount = 0;

(function(obj) {
  assert.sameValue(obj.a, 1);
  assert.sameValue(obj.b, 2);
  assert.sameValue(Object.keys(obj).length, 2);
  callCount += 1;
}({a: 1, b: 2, ...undefined}));

assert.sameValue(callCount, 1);
