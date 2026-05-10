// This file was procedurally generated from the following sources:
// - src/spread/obj-manipulate-outter-obj-in-getter.case
// - src/spread/default/array.template
/*---
description: Getter manipulates outter object before it's spread operation (Array initializer)
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

    Pending Runtime Semantics: PropertyDefinitionEvaluation

    PropertyDefinition:...AssignmentExpression

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let fromValue be GetValue(exprValue).
    3. ReturnIfAbrupt(fromValue).
    4. Let excludedNames be a new empty List.
    5. Return CopyDataProperties(object, fromValue, excludedNames).

---*/
var o = { a: 0, b: 1 };
var cthulhu = { get x() {
  delete o.a;
  o.b = 42;
  o.c = "ni";
}};

var callCount = 0;

(function(obj) {
  assert.sameValue(obj.hasOwnProperty("a"), false);
  assert.sameValue(obj.b, 42);
  assert.sameValue(obj.c, "ni");
  assert(obj.hasOwnProperty("x"));
  assert.sameValue(Object.keys(obj).length, 3);
  callCount += 1;
}.apply(null, [{...cthulhu, ...o}]));

assert.sameValue(callCount, 1);
