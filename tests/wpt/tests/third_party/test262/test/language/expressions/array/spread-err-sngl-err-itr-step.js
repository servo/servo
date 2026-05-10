// This file was procedurally generated from the following sources:
// - src/spread/sngl-err-itr-step.case
// - src/spread/error/array.template
/*---
description: Spread operator applied to the only argument when IteratorStep fails (Array initializer)
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

assert.throws(Test262Error, function() {
  [...iter];
});
