// This file was procedurally generated from the following sources:
// - src/spread/obj-spread-order.case
// - src/spread/default/member-expr.template
/*---
description: Spread operation follows [[OwnPropertyKeys]] order (`new` operator)
esid: sec-new-operator-runtime-semantics-evaluation
features: [Symbol, object-spread]
flags: [generated]
includes: [compareArray.js]
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
var calls = [];
var o = { get z() { calls.push('z') }, get a() { calls.push('a') } };
Object.defineProperty(o, 1, { get: () => { calls.push(1) }, enumerable: true });
Object.defineProperty(o, Symbol('foo'), { get: () => { calls.push("Symbol(foo)") }, enumerable: true });


var callCount = 0;

new function(obj) {
  assert.compareArray(calls, [1, 'z', 'a', "Symbol(foo)"]);
  assert.sameValue(Object.keys(obj).length, 3);
  callCount += 1;
}({...o});

assert.sameValue(callCount, 1);
