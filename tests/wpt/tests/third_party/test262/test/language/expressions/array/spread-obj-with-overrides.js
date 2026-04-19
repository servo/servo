// This file was procedurally generated from the following sources:
// - src/spread/obj-with-overrides.case
// - src/spread/default/array.template
/*---
description: Object Spread properties being overriden (Array initializer)
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
let o = {a: 2, b: 3, c: 4, e: undefined, f: null, g: false};


var callCount = 0;

(function(obj) {
  assert.sameValue(obj.a, 1);
  assert.sameValue(obj.b, 7);
  assert.sameValue(obj.c, 4);
  assert.sameValue(obj.d, 5);
  assert(obj.hasOwnProperty("e"));
  assert.sameValue(obj.f, null);
  assert.sameValue(obj.g, false);
  assert.sameValue(obj.h, -0);
  assert.sameValue(obj.i.toString(), "Symbol(foo)");
  assert(Object.is(obj.j, o));
  assert.sameValue(Object.keys(obj).length, 10);
  callCount += 1;
}.apply(null, [{...o, a: 1, b: 7, d: 5, h: -0, i: Symbol("foo"), j: o}]));

assert.sameValue(callCount, 1);
