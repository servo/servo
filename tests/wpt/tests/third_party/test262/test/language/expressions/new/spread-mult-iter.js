// This file was procedurally generated from the following sources:
// - src/spread/mult-iter.case
// - src/spread/default/member-expr.template
/*---
description: Spread operator following other arguments with a valid iterator (`new` operator)
esid: sec-new-operator-runtime-semantics-evaluation
features: [Symbol.iterator]
flags: [generated]
info: |
    MemberExpression : new MemberExpression Arguments

    1. Return EvaluateNew(MemberExpression, Arguments).

    12.3.3.1.1 Runtime Semantics: EvaluateNew

    6. If arguments is empty, let argList be an empty List.
    7. Else,
       a. Let argList be ArgumentListEvaluation of arguments.
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
  var nextCount = 3;
  return {
    next: function() {
      nextCount += 1;
      return { done: nextCount === 6, value: nextCount };
    }
  };
};

var callCount = 0;

new function() {
  assert.sameValue(arguments.length, 5);
  assert.sameValue(arguments[0], 1);
  assert.sameValue(arguments[1], 2);
  assert.sameValue(arguments[2], 3);
  assert.sameValue(arguments[3], 4);
  assert.sameValue(arguments[4], 5);
  callCount += 1;
}(1, 2, 3, ...iter);

assert.sameValue(callCount, 1);
