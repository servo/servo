// This file was procedurally generated from the following sources:
// - src/spread/mult-err-iter-get-value.case
// - src/spread/error/super-call.template
/*---
description: Spread operator following other arguments when GetIterator fails (@@iterator function return value) (SuperCall)
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

    ArgumentList : ArgumentList , ... AssignmentExpression

    1. Let precedingArgs be the result of evaluating ArgumentList.
    2. Let spreadRef be the result of evaluating AssignmentExpression.
    3. Let iterator be GetIterator(GetValue(spreadRef) ).
    4. ReturnIfAbrupt(iterator).

    7.4.1 GetIterator ( obj, method )

    [...]
    2. Let iterator be ? Call(method, obj).
    3. If Type(iterator) is not Object, throw a TypeError exception.
---*/
var iter = {};
Object.defineProperty(iter, Symbol.iterator, {
  get: function() {
    return null;
  }
});

class Test262ParentClass {
  constructor() {}
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super(0, ...iter);
  }
}

assert.throws(TypeError, function() {
  new Test262ChildClass();
});
