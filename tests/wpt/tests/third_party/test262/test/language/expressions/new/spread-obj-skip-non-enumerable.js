// This file was procedurally generated from the following sources:
// - src/spread/obj-skip-non-enumerable.case
// - src/spread/default/member-expr.template
/*---
description: Object Spread doesn't copy non-enumerable properties (`new` operator)
esid: sec-new-operator-runtime-semantics-evaluation
features: [object-spread]
flags: [generated]
info: |
    MemberExpression : new MemberExpression Arguments

    1. Return EvaluateNew(MemberExpression, Arguments).

    12.3.3.1.1 Runtime Semantics: EvaluateNew

    6. If arguments is empty, let argList be an empty List.
    7. Else,
       a. Let argList be ArgumentListEvaluation of arguments.
       [...]
---*/

let o = {};
Object.defineProperty(o, "b", {value: 3, enumerable: false});


var callCount = 0;

new function(obj) {
  assert.sameValue(obj.hasOwnProperty("b"), false)
  assert.sameValue(Object.keys(obj).length, 0);
  callCount += 1;
}({...o});

assert.sameValue(callCount, 1);
