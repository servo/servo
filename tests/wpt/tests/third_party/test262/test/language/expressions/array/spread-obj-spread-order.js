// This file was procedurally generated from the following sources:
// - src/spread/obj-spread-order.case
// - src/spread/default/array.template
/*---
description: Spread operation follows [[OwnPropertyKeys]] order (Array initializer)
esid: sec-runtime-semantics-arrayaccumulation
features: [Symbol, object-spread]
flags: [generated]
includes: [compareArray.js]
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

    Pending Runtime Semantics: PropertyDefinitionEvaluation

    PropertyDefinition:...AssignmentExpression

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let fromValue be GetValue(exprValue).
    3. ReturnIfAbrupt(fromValue).
    4. Let excludedNames be a new empty List.
    5. Return CopyDataProperties(object, fromValue, excludedNames).

---*/
var calls = [];
var o = { get z() { calls.push('z') }, get a() { calls.push('a') } };
Object.defineProperty(o, 1, { get: () => { calls.push(1) }, enumerable: true });
Object.defineProperty(o, Symbol('foo'), { get: () => { calls.push("Symbol(foo)") }, enumerable: true });


var callCount = 0;

(function(obj) {
  assert.compareArray(calls, [1, 'z', 'a', "Symbol(foo)"]);
  assert.sameValue(Object.keys(obj).length, 3);
  callCount += 1;
}.apply(null, [{...o}]));

assert.sameValue(callCount, 1);
