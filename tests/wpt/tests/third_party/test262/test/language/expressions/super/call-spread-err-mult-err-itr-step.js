// This file was procedurally generated from the following sources:
// - src/spread/mult-err-itr-step.case
// - src/spread/error/super-call.template
/*---
description: Spread operator following other arguments when IteratorStep fails (SuperCall)
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

    7.4.5 IteratorStep ( iterator )

    1. Let result be IteratorNext(iterator).
    2. ReturnIfAbrupt(result).

    7.4.2 IteratorNext ( iterator, value )

    1. If value was not passed, then
       a. Let result be Invoke(iterator, "next", « »).
    [...]
    3. ReturnIfAbrupt(result).
---*/
var iter = {};
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      throw new Test262Error();
    }
  };
};

class Test262ParentClass {
  constructor() {}
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super(0, ...iter);
  }
}

assert.throws(Test262Error, function() {
  new Test262ChildClass();
});
