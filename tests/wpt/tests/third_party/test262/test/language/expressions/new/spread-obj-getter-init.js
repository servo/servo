// This file was procedurally generated from the following sources:
// - src/spread/obj-getter-init.case
// - src/spread/default/member-expr.template
/*---
description: Getter in object literal is not evaluated (`new` operator)
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

let o = {a: 2, b: 3};
let executedGetter = false;


var callCount = 0;

new function(obj) {
  assert.sameValue(obj.a, 2);
  assert.sameValue(obj.b, 3);
  assert.sameValue(executedGetter, false)
  assert.sameValue(Object.keys(obj).length, 3);
  callCount += 1;
}({...o, get c() { executedGetter = true; }});

assert.sameValue(callCount, 1);
