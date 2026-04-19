// This file was procedurally generated from the following sources:
// - src/spread/sngl-err-itr-get-value.case
// - src/spread/error/super-call.template
/*---
description: Spread operator applied to the only argument when GetIterator fails (@@iterator function return value) (SuperCall)
esid: sec-super-keyword-runtime-semantics-evaluation
features: [Symbol.iterator]
flags: [generated]
info: |
    SuperCall : super Arguments

    1. Let newTarget be GetNewTarget().
    2. If newTarget is undefined, throw a ReferenceError exception.
    3. Let func be GetSuperConstructor().
    4. ReturnIfAbrupt(func).
    5. Let argList be ArgumentListEvaluation of Arguments.
    [...]

    12.3.6.1 Runtime Semantics: ArgumentListEvaluation

    ArgumentList : ... AssignmentExpression

    1. Let list be an empty List.
    2. Let spreadRef be the result of evaluating AssignmentExpression.
    3. Let spreadObj be GetValue(spreadRef).
    4. Let iterator be GetIterator(spreadObj).
    5. ReturnIfAbrupt(iterator).

    7.4.1 GetIterator ( obj, method )

    [...]
    2. Let iterator be ? Call(method, obj).
    3. If Type(iterator) is not Object, throw a TypeError exception.
---*/
var iter = {};
iter[Symbol.iterator] = function() {
  return null;
};

class Test262ParentClass {
  constructor() {}
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super(...iter);
  }
}

assert.throws(TypeError, function() {
  new Test262ChildClass();
});
