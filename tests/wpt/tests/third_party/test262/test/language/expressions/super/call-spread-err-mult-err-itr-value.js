// This file was procedurally generated from the following sources:
// - src/spread/mult-err-itr-value.case
// - src/spread/error/super-call.template
/*---
description: Spread operator following other arguments when IteratorValue fails (SuperCall)
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

    7.4.4 IteratorValue ( iterResult )

    1. Assert: Type(iterResult) is Object.
    2. Return Get(iterResult, "value").

    7.3.1 Get (O, P)

    [...]
    3. Return O.[[Get]](P, O).
---*/
var iter = {};
var poisonedValue = Object.defineProperty({}, 'value', {
  get: function() {
    throw new Test262Error();
  }
});
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      return poisonedValue;
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
