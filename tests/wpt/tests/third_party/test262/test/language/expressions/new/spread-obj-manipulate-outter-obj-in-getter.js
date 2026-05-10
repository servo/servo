// This file was procedurally generated from the following sources:
// - src/spread/obj-manipulate-outter-obj-in-getter.case
// - src/spread/default/member-expr.template
/*---
description: Getter manipulates outter object before it's spread operation (`new` operator)
esid: sec-new-operator-runtime-semantics-evaluation
features: [object-spread]
flags: [generated]
info: |
    MemberExpression : new MemberExpression Arguments

    1. Return EvaluateNew(MemberExpression, Arguments).

    12.3.3.1.1 Runtime Semantics: EvaluateNew

    6. If arguments is empty, let argList be an empty List.
    7. Else,
       a. Let argList be ArgumentListEvaluation of arguments.
       [...]

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

new function(obj) {
  assert.sameValue(obj.hasOwnProperty("a"), false);
  assert.sameValue(obj.b, 42);
  assert.sameValue(obj.c, "ni");
  assert(obj.hasOwnProperty("x"));
  assert.sameValue(Object.keys(obj).length, 3);
  callCount += 1;
}({...cthulhu, ...o});

assert.sameValue(callCount, 1);
