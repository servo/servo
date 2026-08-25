// This file was procedurally generated from the following sources:
// - src/spread/obj-override-immutable.case
// - src/spread/default/super-call.template
/*---
description: Object Spread overriding immutable properties (SuperCall)
esid: sec-super-keyword-runtime-semantics-evaluation
features: [object-spread]
flags: [generated]
includes: [propertyHelper.js]
info: |
    SuperCall : super Arguments

    1. Let newTarget be GetNewTarget().
    2. If newTarget is undefined, throw a ReferenceError exception.
    3. Let func be GetSuperConstructor().
    4. ReturnIfAbrupt(func).
    5. Let argList be ArgumentListEvaluation of Arguments.
    [...]
---*/

let o = {b: 2};
Object.defineProperty(o, "a", {value: 1, enumerable: true, writable: false, configurable: true});


var callCount = 0;

class Test262ParentClass {
  constructor(obj) {
    assert.sameValue(obj.a, 3)
    assert.sameValue(obj.b, 2);

    verifyProperty(obj, "a", {
      enumerable: true,
      writable: true,
      configurable: true,
      value: 3
    });

    verifyProperty(obj, "b", {
      enumerable: true,
      writable: true,
      configurable: true,
      value: 2
    });
    callCount += 1;
  }
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super({...o, a: 3});
  }
}

new Test262ChildClass();
assert.sameValue(callCount, 1);
