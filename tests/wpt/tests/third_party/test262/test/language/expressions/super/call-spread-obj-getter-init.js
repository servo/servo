// This file was procedurally generated from the following sources:
// - src/spread/obj-getter-init.case
// - src/spread/default/super-call.template
/*---
description: Getter in object literal is not evaluated (SuperCall)
esid: sec-super-keyword-runtime-semantics-evaluation
features: [object-spread]
flags: [generated]
info: |
    SuperCall : super Arguments

    1. Let newTarget be GetNewTarget().
    2. If newTarget is undefined, throw a ReferenceError exception.
    3. Let func be GetSuperConstructor().
    4. ReturnIfAbrupt(func).
    5. Let argList be ArgumentListEvaluation of Arguments.
    [...]
---*/

let o = {a: 2, b: 3};
let executedGetter = false;


var callCount = 0;

class Test262ParentClass {
  constructor(obj) {
    assert.sameValue(obj.a, 2);
    assert.sameValue(obj.b, 3);
    assert.sameValue(executedGetter, false)
    assert.sameValue(Object.keys(obj).length, 3);
    callCount += 1;
  }
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super({...o, get c() { executedGetter = true; }});
  }
}

new Test262ChildClass();
assert.sameValue(callCount, 1);
