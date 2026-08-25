// This file was procedurally generated from the following sources:
// - src/spread/sngl-iter.case
// - src/spread/default/super-call.template
/*---
description: Spread operator applied to the only argument with a valid iterator (SuperCall)
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
    6. Repeat
       a. Let next be IteratorStep(iterator).
       b. ReturnIfAbrupt(next).
       c. If next is false, return list.
       d. Let nextArg be IteratorValue(next).
       e. ReturnIfAbrupt(nextArg).
       f. Append nextArg as the last element of list.
---*/
var iter = {};
iter[Symbol.iterator] = function() {
  var nextCount = 0;
  return {
    next: function() {
      nextCount += 1;
      return { done: nextCount === 3, value: nextCount };
    }
  };
};

var callCount = 0;

class Test262ParentClass {
  constructor() {
    assert.sameValue(arguments.length, 2);
    assert.sameValue(arguments[0], 1);
    assert.sameValue(arguments[1], 2);
    callCount += 1;
  }
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super(...iter);
  }
}

new Test262ChildClass();
assert.sameValue(callCount, 1);
