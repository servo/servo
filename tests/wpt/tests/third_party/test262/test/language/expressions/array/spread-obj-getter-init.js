// This file was procedurally generated from the following sources:
// - src/spread/obj-getter-init.case
// - src/spread/default/array.template
/*---
description: Getter in object literal is not evaluated (Array initializer)
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

let o = {a: 2, b: 3};
let executedGetter = false;


var callCount = 0;

(function(obj) {
  assert.sameValue(obj.a, 2);
  assert.sameValue(obj.b, 3);
  assert.sameValue(executedGetter, false)
  assert.sameValue(Object.keys(obj).length, 3);
  callCount += 1;
}.apply(null, [{...o, get c() { executedGetter = true; }}]));

assert.sameValue(callCount, 1);
