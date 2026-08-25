// This file was procedurally generated from the following sources:
// - src/spread/obj-skip-non-enumerable.case
// - src/spread/default/super-call.template
/*---
description: Object Spread doesn't copy non-enumerable properties (SuperCall)
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

let o = {};
Object.defineProperty(o, "b", {value: 3, enumerable: false});


var callCount = 0;

class Test262ParentClass {
  constructor(obj) {
    assert.sameValue(obj.hasOwnProperty("b"), false)
    assert.sameValue(Object.keys(obj).length, 0);
    callCount += 1;
  }
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super({...o});
  }
}

new Test262ChildClass();
assert.sameValue(callCount, 1);
