// This file was procedurally generated from the following sources:
// - src/spread/obj-getter-descriptor.case
// - src/spread/default/super-call.template
/*---
description: Spread operation with getter results in data property descriptor (SuperCall)
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

class Test262ParentClass {
  constructor(obj) {
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
  }
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super({...o, c: 4, d: 5});
  }
}

new Test262ChildClass();
assert.sameValue(callCount, 1);
