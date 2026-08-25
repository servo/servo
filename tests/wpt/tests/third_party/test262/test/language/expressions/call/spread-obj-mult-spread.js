// This file was procedurally generated from the following sources:
// - src/spread/obj-mult-spread.case
// - src/spread/default/call-expr.template
/*---
description: Multiple Object Spread operation (CallExpression)
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

let o = {a: 2, b: 3};
let o2 = {c: 4, d: 5};


var callCount = 0;

(function(obj) {
  assert.sameValue(obj.a, 2);
  assert.sameValue(obj.b, 3);
  assert.sameValue(obj.c, 4);
  assert.sameValue(obj.d, 5);
  assert.sameValue(Object.keys(obj).length, 4);
  callCount += 1;
}({...o, ...o2}));

assert.sameValue(callCount, 1);
