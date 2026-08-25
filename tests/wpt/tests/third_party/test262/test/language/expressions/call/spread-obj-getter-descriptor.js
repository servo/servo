// This file was procedurally generated from the following sources:
// - src/spread/obj-getter-descriptor.case
// - src/spread/default/call-expr.template
/*---
description: Spread operation with getter results in data property descriptor (CallExpression)
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
let o = {
    get a() {
        return 42;
    }
};


var callCount = 0;

(function(obj) {
  assert.sameValue(obj.c, 4);
  assert.sameValue(obj.d, 5);
  assert.sameValue(Object.keys(obj).length, 3);

  verifyProperty(obj, "a", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 42
  });
  callCount += 1;
}({...o, c: 4, d: 5}));

assert.sameValue(callCount, 1);
