// This file was procedurally generated from the following sources:
// - src/spread/mult-obj-ident.case
// - src/spread/default/array.template
/*---
description: Object Spread operator following other properties (Array initializer)
esid: sec-runtime-semantics-arrayaccumulation
features: [object-spread]
flags: [generated]
includes: [propertyHelper.js]
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
let o = {c: 3, d: 4};


var callCount = 0;

(function(obj) {
  assert.sameValue(Object.keys(obj).length, 4);

  verifyProperty(obj, "a", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 1
  });

  verifyProperty(obj, "b", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 2
  });

  verifyProperty(obj, "c", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 3
  });

  verifyProperty(obj, "d", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 4
  });
  callCount += 1;
}.apply(null, [{a: 1, b: 2, ...o}]));

assert.sameValue(callCount, 1);
