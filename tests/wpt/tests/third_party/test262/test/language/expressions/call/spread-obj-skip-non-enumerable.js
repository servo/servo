// This file was procedurally generated from the following sources:
// - src/spread/obj-skip-non-enumerable.case
// - src/spread/default/call-expr.template
/*---
description: Object Spread doesn't copy non-enumerable properties (CallExpression)
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

let o = {};
Object.defineProperty(o, "b", {value: 3, enumerable: false});


var callCount = 0;

(function(obj) {
  assert.sameValue(obj.hasOwnProperty("b"), false)
  assert.sameValue(Object.keys(obj).length, 0);
  callCount += 1;
}({...o}));

assert.sameValue(callCount, 1);
