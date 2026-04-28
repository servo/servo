// This file was procedurally generated from the following sources:
// - src/spread/obj-with-overrides.case
// - src/spread/default/super-call.template
/*---
description: Object Spread properties being overriden (SuperCall)
esid: sec-super-keyword-runtime-semantics-evaluation
features: [Symbol, object-spread]
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
let o = {a: 2, b: 3, c: 4, e: undefined, f: null, g: false};


var callCount = 0;

class Test262ParentClass {
  constructor(obj) {
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
  }
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super({...o, a: 1, b: 7, d: 5, h: -0, i: Symbol("foo"), j: o});
  }
}

new Test262ChildClass();
assert.sameValue(callCount, 1);
