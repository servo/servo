// This file was procedurally generated from the following sources:
// - src/spread/obj-skip-non-enumerable.case
// - src/spread/default/array.template
/*---
description: Object Spread doesn't copy non-enumerable properties (Array initializer)
esid: sec-runtime-semantics-arrayaccumulation
features: [object-spread]
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
---*/

let o = {};
Object.defineProperty(o, "b", {value: 3, enumerable: false});


var callCount = 0;

(function(obj) {
  assert.sameValue(obj.hasOwnProperty("b"), false)
  assert.sameValue(Object.keys(obj).length, 0);
  callCount += 1;
}.apply(null, [{...o}]));

assert.sameValue(callCount, 1);
