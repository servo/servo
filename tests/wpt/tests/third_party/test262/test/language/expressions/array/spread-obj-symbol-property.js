// This file was procedurally generated from the following sources:
// - src/spread/obj-symbol-property.case
// - src/spread/default/array.template
/*---
description: Spread operation where source object contains Symbol properties (Array initializer)
esid: sec-runtime-semantics-arrayaccumulation
features: [Symbol, object-spread]
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

    Pending Runtime Semantics: PropertyDefinitionEvaluation

    PropertyDefinition:...AssignmentExpression

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let fromValue be GetValue(exprValue).
    3. ReturnIfAbrupt(fromValue).
    4. Let excludedNames be a new empty List.
    5. Return CopyDataProperties(object, fromValue, excludedNames).

---*/
let symbol = Symbol('foo');
let o = {};
o[symbol] = 1;


var callCount = 0;

(function(obj) {
  assert.sameValue(obj[symbol], 1);
  assert(Object.prototype.hasOwnProperty.call(obj, symbol), "symbol is an own property");
  assert.sameValue(obj.c, 4);
  assert.sameValue(obj.d, 5);
  assert.sameValue(Object.keys(obj).length, 2);
  callCount += 1;
}.apply(null, [{...o, c: 4, d: 5}]));

assert.sameValue(callCount, 1);
