// This file was procedurally generated from the following sources:
// - src/spread/obj-with-overrides.case
// - src/spread/default/call-expr.template
/*---
description: Object Spread properties being overriden (CallExpression)
esid: sec-function-calls-runtime-semantics-evaluation
features: [Symbol, object-spread]
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
let o = {a: 2, b: 3, c: 4, e: undefined, f: null, g: false};


var callCount = 0;

(function(obj) {
  assert.sameValue(obj.a, 1);
  assert.sameValue(obj.b, 7);
  assert.sameValue(obj.c, 4);
  assert.sameValue(obj.d, 5);
  assert(obj.hasOwnProperty("e"));
  assert.sameValue(obj.f, null);
  assert.sameValue(obj.g, false);
  assert.sameValue(obj.h, -0);
  assert.sameValue(obj.i.toString(), "Symbol(foo)");
  assert(Object.is(obj.j, o));
  assert.sameValue(Object.keys(obj).length, 10);
  callCount += 1;
}({...o, a: 1, b: 7, d: 5, h: -0, i: Symbol("foo"), j: o}));

assert.sameValue(callCount, 1);
