// This file was procedurally generated from the following sources:
// - src/spread/obj-spread-order.case
// - src/spread/default/call-expr.template
/*---
description: Spread operation follows [[OwnPropertyKeys]] order (CallExpression)
esid: sec-function-calls-runtime-semantics-evaluation
features: [Symbol, object-spread]
flags: [generated]
includes: [compareArray.js]
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
var calls = [];
var o = { get z() { calls.push('z') }, get a() { calls.push('a') } };
Object.defineProperty(o, 1, { get: () => { calls.push(1) }, enumerable: true });
Object.defineProperty(o, Symbol('foo'), { get: () => { calls.push("Symbol(foo)") }, enumerable: true });


var callCount = 0;

(function(obj) {
  assert.compareArray(calls, [1, 'z', 'a', "Symbol(foo)"]);
  assert.sameValue(Object.keys(obj).length, 3);
  callCount += 1;
}({...o}));

assert.sameValue(callCount, 1);
