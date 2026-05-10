// This file was procedurally generated from the following sources:
// - src/spread/obj-getter-init.case
// - src/spread/default/call-expr.template
/*---
description: Getter in object literal is not evaluated (CallExpression)
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
---*/

let o = {a: 2, b: 3};
let executedGetter = false;


var callCount = 0;

(function(obj) {
  assert.sameValue(obj.a, 2);
  assert.sameValue(obj.b, 3);
  assert.sameValue(executedGetter, false)
  assert.sameValue(Object.keys(obj).length, 3);
  callCount += 1;
}({...o, get c() { executedGetter = true; }}));

assert.sameValue(callCount, 1);
