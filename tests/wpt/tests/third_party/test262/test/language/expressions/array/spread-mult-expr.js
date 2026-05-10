// This file was procedurally generated from the following sources:
// - src/spread/mult-expr.case
// - src/spread/default/array.template
/*---
description: Spread operator applied to AssignmentExpression following other elements (Array initializer)
esid: sec-runtime-semantics-arrayaccumulation
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

    ArgumentList : ArgumentList , ... AssignmentExpression

    1. Let precedingArgs be the result of evaluating ArgumentList.
    2. Let spreadRef be the result of evaluating AssignmentExpression.
    3. Let iterator be GetIterator(GetValue(spreadRef) ).
    4. ReturnIfAbrupt(iterator).
    5. Repeat
       a. Let next be IteratorStep(iterator).
       b. ReturnIfAbrupt(next).
       c. If next is false, return precedingArgs.
---*/
var source = [3, 4, 5];
var target;

var callCount = 0;

(function() {
  assert.sameValue(arguments.length, 5);
  assert.sameValue(arguments[0], 1);
  assert.sameValue(arguments[1], 2);
  assert.sameValue(arguments[2], 3);
  assert.sameValue(arguments[3], 4);
  assert.sameValue(arguments[4], 5);
  assert.sameValue(target, source);
  callCount += 1;
}.apply(null, [1, 2, ...target = source]));

assert.sameValue(callCount, 1);
