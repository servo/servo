// This file was procedurally generated from the following sources:
// - src/spread/sngl-iter.case
// - src/spread/default/array.template
/*---
description: Spread operator applied to the only argument with a valid iterator (Array initializer)
esid: sec-runtime-semantics-arrayaccumulation
features: [Symbol.iterator]
flags: [generated]
info: |
    SpreadElement : ...AssignmentExpression

    1. Let spreadRef be the result of evaluating AssignmentExpression.
    2. Let spreadObj be ? GetValue(spreadRef).
    3. Let iterator be ? GetIterator(spreadObj).
    4. Repeat
       a. Let next be ? IteratorStep(iterator).
       b. If next is false, return nextIndex.
       c. Let nextValue be ? IteratorValue(next).
       d. Let status be CreateDataProperty(array, ToString(ToUint32(nextIndex)),
          nextValue).
       e. Assert: status is true.
       f. Let nextIndex be nextIndex + 1.

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

(function() {
  assert.sameValue(arguments.length, 2);
  assert.sameValue(arguments[0], 1);
  assert.sameValue(arguments[1], 2);
  callCount += 1;
}.apply(null, [...iter]));

assert.sameValue(callCount, 1);
