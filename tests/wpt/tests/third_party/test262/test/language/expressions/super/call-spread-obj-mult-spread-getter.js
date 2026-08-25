// This file was procedurally generated from the following sources:
// - src/spread/obj-mult-spread-getter.case
// - src/spread/default/super-call.template
/*---
description: Multiple Object Spread usage calls getter multiple times (SuperCall)
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

    Pending Runtime Semantics: PropertyDefinitionEvaluation

    PropertyDefinition:...AssignmentExpression

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let fromValue be GetValue(exprValue).
    3. ReturnIfAbrupt(fromValue).
    4. Let excludedNames be a new empty List.
    5. Return CopyDataProperties(object, fromValue, excludedNames).

---*/
let getterCallCount = 0;
let o = {
    get a() {
        return ++getterCallCount;
    }
};


var callCount = 0;

class Test262ParentClass {
  constructor(obj) {
    assert.sameValue(obj.a, 2);
    assert.sameValue(obj.c, 4);
    assert.sameValue(obj.d, 5);
    assert.sameValue(Object.keys(obj).length, 3);
    callCount += 1;
  }
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super({...o, c: 4, d: 5, a: 42, ...o});
  }
}

new Test262ChildClass();
assert.sameValue(callCount, 1);
